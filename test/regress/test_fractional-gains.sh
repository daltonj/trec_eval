# Gains 0.9,0.3 make the run ideal. The gain table must reverse grade
# order even though subtracting these gains truncates to 0 as an int.
. test/regress/lib.sh
printf '1 0 d1 1\n1 0 d2 2\n' > "$T/qrels"
expect_output test/regress/fractional-gains/out.ideal -q -m ndcg.1=0.9,2=0.3 \
    "$T/qrels" test/regress/results
