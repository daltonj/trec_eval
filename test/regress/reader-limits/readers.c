/* Checks the readers' handling of file sizes beyond an int or unsigned and
   of failed allocations on standard input, with small inputs, and the
   searches of the included convert_zscores.c. The make rule renames ftell,
   malloc and realloc to the functions below: ftell reports a size of
   UINT_MAX + 1, malloc refuses a request above UINT_MAX, or every request
   while refuse_malloc is set, and realloc refuses every request. Each
   records the size it refuses. */
#include <limits.h>
#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"
#include "convert_zscores.c"

int te_get_qrels(EPI * epi, char *text_qrels_file, ALL_REL_INFO * all_rel_info);
int te_get_qrels_jg(EPI * epi, char *text_qrels_file,
                    ALL_REL_INFO * all_rel_info);
int te_get_prefs(EPI * epi, char *text_prefs_file, ALL_REL_INFO * all_rel_info);
int te_get_qrels_prefs(EPI * epi, char *text_prefs_file,
                       ALL_REL_INFO * all_rel_info);
int te_get_trec_results(EPI * epi, char *text_results_file,
                        ALL_RESULTS * all_results);

static size_t refused;
static int refuse_malloc;

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

#if LONG_MAX > UINT_MAX
static int check_file_size(const char *reader, int status)
{
    int ok = status == UNDEF && refused == (size_t) UINT_MAX + 3;

    if (!ok)
        printf("%s did not ask for a buffer of the whole file\n", reader);
    refused = 0;
    return (!ok);
}
#endif

static int check_file_sizes(char *path)
{
#if LONG_MAX > UINT_MAX
    int failed = 0;
    EPI epi = { 0 };
    ALL_REL_INFO rel_info = { 0 };
    ALL_ZSCORES zscores = { 0 };

    /* UINT_MAX + 1 crosses both the int and unsigned size boundaries.
       Each reader needs two additional bytes for a newline and NUL. */
    failed |= check_file_size("te_get_qrels",
                              te_get_qrels(&epi, path, &rel_info));
    failed |= check_file_size("te_get_qrels_jg",
                              te_get_qrels_jg(&epi, path, &rel_info));
    failed |= check_file_size("te_get_prefs",
                              te_get_prefs(&epi, path, &rel_info));
    failed |= check_file_size("te_get_qrels_prefs",
                              te_get_qrels_prefs(&epi, path, &rel_info));
    failed |= check_file_size("te_get_zscores",
                              te_get_zscores(&epi, path, &zscores));
    return (failed);
#else
    (void) path;
    puts("SKIP file sizes: UINT_MAX + 1 needs a wider long and ftell");
    return (0);
#endif
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

static int check_stdin(char *path, int initial)
{
    EPI epi = { 0 };
    ALL_RESULTS results = { 0 };
    size_t expected = initial ? 4096 : 8192;
    int status;

    if (NULL == freopen(path, "rb", stdin)) {
        perror(path);
        return (1);
    }
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
    if (argc == 2 && 0 == strcmp(argv[1], "zscores"))
        return (check_zscores());
    if (argc == 3) {
        if (0 == strcmp(argv[1], "file-sizes"))
            return (check_file_sizes(argv[2]));
        if (0 == strcmp(argv[1], "stdin-malloc"))
            return (check_stdin(argv[2], 1));
        if (0 == strcmp(argv[1], "stdin-realloc"))
            return (check_stdin(argv[2], 0));
    }
    fprintf(stderr, "usage: %s zscores | "
            "{file-sizes|stdin-malloc|stdin-realloc} FILE\n", argv[0]);
    return (1);
}
