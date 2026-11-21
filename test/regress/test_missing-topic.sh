# Missing qid 2 is reported before any query output. -D selects qid 1;
# result qid 3, absent from the qrels, is ignored.
. test/regress/lib.sh
D=test/regress/missing-topic

expect_result 4 /dev/null $D/err -q -m num_ret $D/qrels $D/results
expect_output $D/out.D -D 0.1 -q -m num_ret $D/qrels $D/results
expect_result 0 $D/out.extra /dev/null -q -m num_ret \
    $D/qrels $D/results.extra
