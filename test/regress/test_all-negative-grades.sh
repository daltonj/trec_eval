# A topic whose grades are all negative has no relevant documents, so its
# average precision is 0. In both fixtures the other topic retrieves its
# one grade 1 document at rank 1 (AP 1), so map is (0 + 1) / 2 = 0.5.
. test/regress/lib.sh
D=test/regress/all-negative-grades

# The all-negative topic is evaluated first.
expect_output test/regress/complete-empty-ranking/out.first -q -m map $D/qrels.first $D/results.first

# The all-negative topic follows a topic with grades 0 and 1.
expect_output $D/out.second -q -m map $D/qrels.second $D/results.second

# The ideal ranking of the all-negative topic has no document, so ndcg_cut
# finishes at once even with a cutoff of 2147483647; the CPU limit fails a run
# that counts up to the cutoff.
(ulimit -t 2
 expect_output $D/out.ndcg-cut -q -m ndcg_cut.2147483647 $D/qrels.first $D/results.first) || exit 1
