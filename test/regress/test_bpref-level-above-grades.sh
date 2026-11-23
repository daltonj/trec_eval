# No grade reaches the relevance level. The nonrelevant-count loop must
# stop at the grade table's end, giving 0 for both preference measures.
. test/regress/lib.sh
expect_output test/regress/bpref-level-above-grades/out.zero -l 2147483647 \
    -m bpref -m gm_bpref test/regress/qrels.one test/regress/results.one
