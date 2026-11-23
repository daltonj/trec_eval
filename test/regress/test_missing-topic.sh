# Without -c, qrels topic 2, which the results lack, is reported before any
# query output. -D selecting topic 1 avoids the error, and results topic 1,
# which the qrels lack, is skipped before topic 2.
. test/regress/lib.sh
D=test/regress/missing-topic
echo "trec_eval: qrels qid '2' not found in results" > "$T/err"
expect_result 4 /dev/null "$T/err" -q -m num_ret \
    test/regress/qrels test/regress/results.one
expect_output $D/out.D -D 0.1 -q -m num_ret \
    test/regress/qrels test/regress/results.one
printf '2 0 d1 1\n' > "$T/qrels"
expect_result 0 $D/out.extra /dev/null -q -m num_ret "$T/qrels" test/regress/results
