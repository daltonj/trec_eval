# The -c recount must select the same query as the scoring loop, including
# a selected qrels topic missing from the run.
. test/regress/lib.sh
D=test/regress/num-rel-debug-query
Q=test/regress/num-rel-complete-level/qrels
R=test/regress/num-rel-complete-level/results
for q in 1 3; do
    expect_output $D/out.$q -c -q -l 2 -D1.$q -m num_rel $Q $R
done
