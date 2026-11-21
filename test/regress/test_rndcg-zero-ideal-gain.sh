# All relevant documents have gain 0, so Rndcg is 0 rather than NaN.
. test/regress/lib.sh
D=test/regress/rndcg-zero-ideal-gain
expect_output $D/out.zero -m Rndcg.1=0 $D/qrels $D/results
