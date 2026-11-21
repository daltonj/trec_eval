# Under -c, the all row of num_rel counts the relevant documents of every
# topic in the qrels, and must use the same relevance level (-l) as the
# topic rows. Topic 3 has no retrieved documents.
. test/regress/lib.sh
D=test/regress/num-rel-complete-level
expect_output $D/out.l2 -c -l 2 -q -m num_rel $D/qrels $D/results
expect_output $D/out.l0 -c -l 0 -q -m num_rel $D/qrels $D/results
expect_output $D/out.l1 -c -q -m num_rel $D/qrels $D/results
