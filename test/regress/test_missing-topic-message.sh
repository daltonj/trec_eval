# Topic 2 of the qrels has no results. Without -c, trec_eval exits 4 with
# a message naming that qrels topic. A results topic that the qrels lack
# (topic 3 of results.extra) is skipped without a message.
. test/regress/lib.sh
D=test/regress/missing-topic-message

expect_stderr 4 $D/err -m num_ret $D/qrels $D/results
expect_stderr 0 /dev/null -m num_ret $D/qrels $D/results.extra
