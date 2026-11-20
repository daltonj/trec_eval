# Measure parameters that are NaN or infinite are refused with status 2:
# cutoffs, parameter lists and name=value pairs.
. test/regress/lib.sh
Q=test/qrels.test
R=test/results.test

for m in iprec_at_recall.nan iprec_at_recall..5,nan,nan set_F.nan ndcg.1=nan; do
    expect_message 2 "value \`nan' is not a number" -m $m $Q $R
done
for m in Rprec_mult.inf 11pt_avg.-inf set_F.inf relstring.infinity \
         utility.inf,-1,0,0 rbp.p=inf rbp_resid.p=-inf; do
    expect_message 2 "is not finite" -m $m $Q $R
done
