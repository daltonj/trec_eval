# Topic 2 of the qrels has no results. Without -c, trec_eval exits 4 with
# a message naming that topic before it evaluates any topic, so -q prints
# no rows for topic 1. -D 0.1 selects topic 1 alone. A results topic that the qrels lack, topic 4 of
# results.extra, is skipped without a message.
. test/regress/lib.sh
D=test/regress/missing-topic

expect_result 4 /dev/null $D/err -q -m num_ret $D/qrels $D/results
expect_output $D/out.D -D 0.1 -q -m num_ret $D/qrels $D/results
expect_result 0 $D/out.extra /dev/null -q -m num_ret \
    $D/qrels $D/results.extra
