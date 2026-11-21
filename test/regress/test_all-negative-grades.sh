# An all-negative topic has AP 0. It comes first to exercise initial grade
# table allocation; the other topic has AP 1, so map=(0+1)/2=0.5.
. test/regress/lib.sh
D=test/regress/all-negative-grades
expect_output $D/out -q -m map $D/qrels $D/results
# The ideal ranking has no document: finish at once even at INT_MAX.
(ulimit -t 2
 expect_output $D/out.ndcg-cut -q -m ndcg_cut.2147483647 $D/qrels $D/results) || exit 1
