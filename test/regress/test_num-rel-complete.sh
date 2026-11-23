# Under -c, num_rel counts every qrels topic at the requested -l, including
# topic 3, which has no results, and only the topic that -D selects.
. test/regress/lib.sh
D=test/regress/num-rel-complete
printf '1 0 d1 1\n1 0 d2 2\n3 0 d1 2\n3 0 d2 0\n' > "$T/qrels"
for level in 0 2; do
    expect_output $D/out.l$level -c -l $level -q -m num_rel \
        "$T/qrels" test/regress/results
done
for q in 1 3; do
    expect_output $D/out.D$q -c -l 2 -q -D1.$q -m num_rel \
        "$T/qrels" test/regress/results
done
