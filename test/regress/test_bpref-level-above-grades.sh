# bpref and gm_bpref count the nonrelevant documents below the relevance
# level. A level above every grade must not read past the table of
# grades, and gives 0 because no document is relevant. The highest grade
# is 2 for topic 1 and 1 for topic 2.
. test/regress/lib.sh
D=test/regress/bpref-level-above-grades
expect_output $D/out.l1 -q -l 1 -m bpref -m gm_bpref $D/qrels $D/results
expect_output $D/out.l2 -q -l 2 -m bpref -m gm_bpref $D/qrels $D/results
expect_output $D/out.zero -q -l 3 -m bpref -m gm_bpref $D/qrels $D/results
expect_output $D/out.zero -q -l 2147483647 -m bpref -m gm_bpref $D/qrels $D/results
