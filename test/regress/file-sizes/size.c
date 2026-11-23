/* Each judgments and zscores reader asks for a buffer of the whole file
   when the size of the file does not fit in an int. The make rule renames
   ftell and malloc: ftell reports a size of INT_MAX + 1 for a small file,
   and an allocation that large is refused, so nothing large is read. */
#include <limits.h>
#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"

int te_get_qrels(EPI * epi, char *text_qrels_file, ALL_REL_INFO * all_rel_info);
int te_get_qrels_jg(EPI * epi, char *text_qrels_file,
                    ALL_REL_INFO * all_rel_info);
int te_get_prefs(EPI * epi, char *text_prefs_file, ALL_REL_INFO * all_rel_info);
int te_get_qrels_prefs(EPI * epi, char *text_prefs_file,
                       ALL_REL_INFO * all_rel_info);

static size_t refused;

long size_ftell(FILE * stream)
{
    (void) stream;
    return ((long) INT_MAX + 1);
}

void *size_malloc(size_t size)
{
    if (size > INT_MAX) {
        refused = size;
        return (NULL);
    }
    return (calloc(size, 1));
}

static int check(const char *reader, int status)
{
    int ok = status == UNDEF && refused == (size_t) INT_MAX + 3;

    if (!ok)
        printf("%s did not ask for a buffer of the whole file\n", reader);
    refused = 0;
    return (!ok);
}

int main(int argc, char **argv)
{
    int failed = 0;
#if LONG_MAX > INT_MAX
    EPI epi = { 0 };
    ALL_REL_INFO rel_info = { 0 };
    ALL_ZSCORES zscores = { 0 };

    failed |= check("te_get_qrels", te_get_qrels(&epi, argv[1], &rel_info));
    failed |= check("te_get_qrels_jg",
                    te_get_qrels_jg(&epi, argv[1], &rel_info));
    failed |= check("te_get_prefs", te_get_prefs(&epi, argv[1], &rel_info));
    failed |= check("te_get_qrels_prefs",
                    te_get_qrels_prefs(&epi, argv[1], &rel_info));
    failed |= check("te_get_zscores",
                    te_get_zscores(&epi, argv[1], &zscores));
#endif
    return (failed);
}
