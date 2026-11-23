/* Checks, with small inputs, that te_read_lines asks for a buffer of a
   whole file whose size is beyond an int or unsigned, that te_fread
   asks for at most TE_READ_CHUNK bytes at a time, that the results
   reader refuses standard input when an allocation fails, and that the
   searches of the included convert_zscores.c stay within their counts.
   The make rule sets TE_READ_CHUNK to 2 and renames ftell, fread, malloc and realloc to the functions below:
   fread records the largest request and reads with getc, ftell reports a size of UINT_MAX + 1, malloc refuses a request above
   UINT_MAX, or every request while refuse_malloc is set, and realloc
   refuses every request. Each records the size it refuses. */
#include <limits.h>
#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"
#include "convert_zscores.c"

int te_get_trec_results(EPI * epi, char *text_results_file,
                        ALL_RESULTS * all_results);

static size_t refused;
static size_t largest_read;
static int refuse_malloc;

size_t reader_fread(void *ptr, size_t size, size_t count, FILE * stream)
{
    size_t n = 0;
    int c;

    largest_read = MAX(largest_read, size * count);
    while (n < size * count && EOF != (c = getc(stream)))
        ((char *) ptr)[n++] = (char) c;
    return (n / size);
}

long reader_ftell(FILE * stream)
{
    (void) stream;
#if LONG_MAX > UINT_MAX
    return ((long) UINT_MAX + 1);
#else
    return (0);
#endif
}

void *reader_malloc(size_t size)
{
    if (size > UINT_MAX || refuse_malloc) {
        refused = size;
        return (NULL);
    }
    return (calloc(size, 1));
}

void *reader_realloc(void *ptr, size_t size)
{
    (void) ptr;
    refused = size;
    return (NULL);
}

static int check_file_size(void)
{
#if LONG_MAX > UINT_MAX
    char *buf;

    /* UINT_MAX + 1 crosses both the int and unsigned size boundaries.
       The buffer needs two more bytes for a newline and NUL. */
    if (UNDEF == te_read_lines("/dev/null", &buf, "test", "test") &&
        refused == (size_t) UINT_MAX + 3)
        return (0);
    puts("te_read_lines did not ask for a buffer of the whole file");
    return (1);
#else
    fputs("SKIP file-size: UINT_MAX + 1 needs a wider long\n", stderr);
    return (0);
#endif
}

static int check_read_chunks(void)
{
    char buf[8];
    size_t got = te_fread(buf, sizeof buf, stdin);

    if (got == 5 && 0 == memcmp(buf, "abcde", 5) && largest_read == 2)
        return (0);
    printf("te_fread read %zu bytes, at most %zu at a time; expected 5, "
           "at most 2\n", got, largest_read);
    return (1);
}

static int check_zscores(void)
{
    /* The matching second entries exist in storage but are outside the
       declared counts. Neither search may inspect those entries. */
    ZSCORES topics[] = { {"a", 0, NULL}, {"z", 0, NULL} };
    ALL_ZSCORES all = { 1, topics };
    ZSCORE_QID measures[] = { {"map", 0, 1}, {"recip_rank", 0, 1} };
    ZSCORES topic = { "a", 1, measures };
    int failed = 0;

    if (NULL != find_qid(&all, "z")) {
        puts("find_qid matched an entry outside num_q_zscores");
        failed = 1;
    }
    if (NULL != find_meas(&topic, "recip_rank")) {
        puts("find_meas matched an entry outside num_zscores");
        failed = 1;
    }
    return (failed);
}

static int check_stdin(int initial)
{
    EPI epi = { 0 };
    ALL_RESULTS results = { 0 };
    size_t expected = initial ? 4096 : 8192;
    int status;

    refuse_malloc = initial;
    status = te_get_trec_results(&epi, "-", &results);
    if (status != UNDEF || refused != expected) {
        printf("stdin: got status %d and allocation %zu; "
               "expected UNDEF and %zu\n", status, refused, expected);
        return (1);
    }
    return (0);
}

int main(int argc, char **argv)
{
    if (argc == 2 && 0 == strcmp(argv[1], "file-size"))
        return (check_file_size());
    if (argc == 2 && 0 == strcmp(argv[1], "read-chunks"))
        return (check_read_chunks());
    if (argc == 2 && 0 == strcmp(argv[1], "stdin-malloc"))
        return (check_stdin(1));
    if (argc == 2 && 0 == strcmp(argv[1], "stdin-realloc"))
        return (check_stdin(0));
    if (argc == 2 && 0 == strcmp(argv[1], "zscores"))
        return (check_zscores());
    fprintf(stderr, "usage: %s file-size | read-chunks | "
            "stdin-malloc | stdin-realloc | zscores\n", argv[0]);
    return (1);
}
