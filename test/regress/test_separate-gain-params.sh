# Independent maps give dcg=3+2/log2(3)=4.2619 and ideal_dcg=5+1/log2(3)
# =5.6309. For ndcg, grades 1,2 have gains 1,4, giving
# (1+4/log2(3))/(4+1/log2(3))=0.7609. Sharing any map changes a result.
. test/regress/lib.sh
D=test/regress/separate-gain-params
expect_output $D/out.each -m dcg.1=3 -m ideal_dcg.2=5 -m ndcg.2=4 \
    $D/qrels test/regress/results
