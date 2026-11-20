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
expect_message 2 "relevance level 1 of measure 'rbp' is given more than once" \
    -m rbp.1=2,p=0.5,+1=3 $Q $R

# Distinct levels, and p beside a level, are accepted.
expect_output $D/out.distinct -m ndcg.0=1,1=3 -m G.0=1,1=3 $Q $R
expect_status 0 -m rbp.p=0.5,2=3 $Q $R

# With both gains equal to 2 and p = 0.5, rbp = 0.5 * (1 + 0.5).
d=test/regress/rbp-gain-range
for params in p=0.5,0=2,1=2 0=2,p=0.5,1=2 0=2,1=2,p=0.5; do
    printf '%-22s\tall\t0.7500\n' "rbp_$params" > "$T/expected"
    expect_result 0 "$T/expected" /dev/null -m rbp.$params \
        $d/qrels.two $d/results.two
done
