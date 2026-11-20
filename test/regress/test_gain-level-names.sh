# A name=value parameter whose name is not a relevance level is refused
# when the measure is initialised, with exit status 2, for every measure
# with such parameters: dcg, ideal_dcg, ndcg, ndcg_rel, Rndcg, G, rbp and
# rbp_resid. A level may have a sign, and rbp and rbp_resid also take p.
. test/regress/lib.sh
Q=test/qrels.test
R=test/results.test

u="Unusable relevance level"
for m in ndcg dcg ideal_dcg ndcg_rel Rndcg G rbp rbp_resid; do
    expect_message 2 "$u 'x'" -m $m.x=1 $Q $R
done
expect_message 2 "$u 'x'" -m ndcg.1=3,x=1 $Q $R
expect_message 2 "$u 'p'" -m ndcg.p=0.5 $Q $R
expect_message 2 "$u '1x'" -m ndcg.1x=2 $Q $R
expect_message 2 "$u ''" -m ndcg.=1 $Q $R
expect_message 2 "$u '99999999999999999999'" \
    -m ndcg.99999999999999999999=1 $Q $R
expect_message 2 "$u 'q'" -m rbp.q=0.5 $Q $R
expect_message 2 "$u 'q'" -m rbp_resid.p=0.5,q=1 $Q $R

for m in ndcg.+1=2 rbp.p=0.5 rbp.p=0.5,2=3 rbp_resid.p=0.5 \
         rbp_resid.p=0.5,2=3; do
    expect_status 0 -m $m $Q $R
done
