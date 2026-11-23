# Topics 1 and 3 have only negative grades, so AP and ideal DCG are 0;
# topic 2 scores 1. Topic 1 comes first, to exercise the initial grade
# table, and topic 3 follows topic 2's grade 1, to exercise its reset.
# Negative grades become RELVALUE_UNJUDGED (-2), given gain 1 so that
# their DCG is nonzero.
. test/regress/lib.sh
D=test/regress/negative-grades
printf '1 0 a -2\n2 0 a 1\n3 0 a -1\n' > "$T/qrels"
printf '1 Q0 a 1 1 r\n2 Q0 a 1 1 r\n3 Q0 a 1 1 r\n' > "$T/results"
expect_output $D/out -q -m map -m ndcg.-2=1 "$T/qrels" "$T/results"
# The ideal ranking has no document: finish at once even at INT_MAX.
(ulimit -t 2
 expect_output $D/out.ndcg-cut -q -m ndcg_cut.2147483647 \
     "$T/qrels" "$T/results") || exit 1
