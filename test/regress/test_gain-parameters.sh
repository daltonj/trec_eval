# Each gain measure validates integer level names, and only rbp and
# rbp_resid take p. Level identity is numeric, so 1 and +1 are duplicates;
# p must also be unique, wherever it appears among the gains.
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
expect_message 2 "relevance level 1 of measure 'ndcg' is given more than once" \
    -m ndcg.2=1,1=2,+1=3 $Q $R
expect_message 2 "parameter 'p' of measure 'rbp' is given more than once" \
    -m rbp.p=0.5,p=0.8 $Q $R
expect_message 2 "relevance level 1 of measure 'rbp' is given more than once" \
    -m rbp.1=2,p=0.5,+1=3 $Q $R
# Signed levels and p alongside a gain are valid. rbp_resid ignores gains.
expect_status 0 -m ndcg.+1=2 -m rbp.p=0.5,2=3 -m rbp_resid.p=0.5,2=3 $Q $R
expect_output test/regress/gain-parameters/out.distinct \
    -m ndcg.0=1,1=3 -m G.0=1,1=3 $Q $R
