# Each gain measure validates integer level names. Only rbp/rbp_resid take p.
. test/regress/lib.sh
Q=test/qrels.test
R=test/results.test
for measure in ndcg dcg ideal_dcg ndcg_rel Rndcg G rbp rbp_resid; do
    expect_message 2 "Unusable relevance level 'x' of measure '$measure'" \
        -m $measure.x=1 $Q $R
done
for level in p 1x '' 99999999999999999999; do
    expect_message 2 "Unusable relevance level '$level' of measure 'ndcg'" \
        -m "ndcg.$level=1" $Q $R
done
# Signed levels and p alongside a gain are valid. rbp_resid ignores gains.
expect_status 0 -m ndcg.+1=2 -m rbp.p=0.5,2=3 -m rbp_resid.p=0.5,2=3 $Q $R
