# -J removes the only retrieved doc, d1, which qrels.abc does not judge.
# Both interpolated precision measures must return 0 rather than 0/0.
. test/regress/lib.sh
expect_output test/regress/eleven-point-no-results/out -J -m 11pt_avg.0 \
    -m iprec_at_recall.0 test/regress/qrels.abc test/regress/results.one
