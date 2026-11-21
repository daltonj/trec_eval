# Gains 0.9,0.3 make the run ideal. The gain table must reverse grade
# order even though subtracting these gains truncates to 0 as an int.
. test/regress/lib.sh
D=test/regress/fractional-gains
expect_output $D/out.ideal -q -m ndcg.1=0.9,2=0.3 $D/qrels $D/results
