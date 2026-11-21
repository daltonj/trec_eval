# A positive gain for unjudged documents has no ideal occurrence. If the
# judged documents all have negative gain, ndcg_rel must stay 0.
. test/regress/lib.sh
D=test/regress/ndcg-rel-zero-ideal-gain

expect_output $D/out.zero -q -m ndcg_rel.-1=1,1=-0.5 $D/qrels $D/results
expect_output $D/out.default -q -m ndcg_rel $D/qrels $D/results
expect_output $D/out.negative -q -m ndcg_rel.-1=1,1=-3 $D/qrels $D/results
