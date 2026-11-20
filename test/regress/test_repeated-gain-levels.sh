# A name=value parameter list that gives the same relevance level twice,
# or the same name twice, is refused with exit status 2. 1, 01 and +1
# are the same level.
. test/regress/lib.sh
D=test/regress/repeated-gain-levels
Q=test/qrels.test
R=test/results.test

for m in ndcg.1=2,1=5 ndcg.1=5,1=2 ndcg.1=2,01=5 ndcg.2=1,1=2,+1=3; do
    expect_message 2 "relevance level 1 of measure 'ndcg' is given more than once" \
        -m $m $Q $R
done
expect_message 2 "relevance level 1 of measure 'G' is given more than once" \
    -m G.1=2,1=3 $Q $R
expect_message 2 "parameter 'p' of measure 'rbp' is given more than once" \
    -m rbp.p=0.5,p=0.8 $Q $R

# Distinct levels, and p beside a level, are accepted.
expect_output $D/out.distinct -m ndcg.0=1,1=3 -m G.0=1,1=3 $Q $R
expect_status 0 -m rbp.p=0.5,2=3 $Q $R
