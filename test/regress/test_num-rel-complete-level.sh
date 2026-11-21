# Under -c, count every qrels topic (3 has no results) at the requested -l.
. test/regress/lib.sh
D=test/regress/num-rel-complete-level
for level in 0 2; do
    expect_output $D/out.l$level -c -l $level -q -m num_rel $D/qrels $D/results
done
