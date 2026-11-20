# Topic 2 of the qrels has no results. Without -c, trec_eval exits 4
# before it evaluates any topic, so -q prints no rows for topic 1. With
# -c, or with -D naming another topic, the output is as before.
. test/regress/lib.sh
D=test/regress/missing-topic-no-output

expect_output_status 4 /dev/null -q -m num_ret $D/qrels $D/results
expect_stderr 4 $D/err -q -m num_ret $D/qrels $D/results

expect_output $D/out.c -c -q -m num_ret $D/qrels $D/results
expect_output $D/out.D -D 0.1 -q -m num_ret $D/qrels $D/results
