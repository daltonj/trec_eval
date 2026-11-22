/* Checks preference rank comparisons without constructing rankings
   with more than INT_MAX documents. Judgment group and relevance
   stay equal so that only rank determines the order. */
#include <limits.h>
#include "form_prefs_counts.c"

static int failed = 0;

static void check(long rank1, long rank2, int want)
{
    DOCNO_RESULTS a = {NULL, 0.0, rank1};
    DOCNO_RESULTS b = {NULL, 0.0, rank2};
    PREFS_AND_RANKS p = {"A", "0", 1.0, NULL, rank1};
    PREFS_AND_RANKS q = {"A", "0", 1.0, NULL, rank2};
    int cmp;

    cmp = comp_results_inc_rank(&a, &b);
    if ((cmp > 0) - (cmp < 0) != want) {
        printf("comp_results_inc_rank(%ld, %ld): got %d, expected sign %d\n",
               rank1, rank2, cmp, want);
        failed = 1;
    }
    cmp = comp_prefs_and_ranks_jg_rel_level(&p, &q);
    if ((cmp > 0) - (cmp < 0) != want) {
        printf("comp_prefs_and_ranks_jg_rel_level(%ld, %ld): "
               "got %d, expected sign %d\n", rank1, rank2, cmp, want);
        failed = 1;
    }
}

int main(void)
{
    check(1, 2, -1);
    check(2, 1, 1);
    check(1, 1, 0);
    /* An unjudged document has rank -1 */
    check(-1, LONG_MAX, -1);
    check(LONG_MAX, -1, 1);
    return (failed);
}
