# Level identity is numeric, so 1 and +1 are duplicates. p is a separate
# parameter and must also be unique, wherever it appears among the gains.
. test/regress/lib.sh
Q=test/qrels.test
R=test/results.test
expect_message 2 "relevance level 1 of measure 'ndcg' is given more than once" \
    -m ndcg.2=1,1=2,+1=3 $Q $R
expect_message 2 "parameter 'p' of measure 'rbp' is given more than once" \
    -m rbp.p=0.5,p=0.8 $Q $R
expect_message 2 "relevance level 1 of measure 'rbp' is given more than once" \
    -m rbp.1=2,p=0.5,+1=3 $Q $R
expect_output test/regress/repeated-gain-levels/out.distinct \
    -m ndcg.0=1,1=3 -m G.0=1,1=3 $Q $R
