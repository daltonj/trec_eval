# -J removes the only retrieved (unjudged) doc. Both interpolated
# precision measures must return 0 rather than 0/0.
. test/regress/lib.sh
printf '1 0 a 1\n' > "$T/qrels"
printf '1 Q0 x 1 1 r\n' > "$T/results"
expect_output test/regress/eleven-point-no-results/out -J \
    -m 11pt_avg.0 -m iprec_at_recall.0 "$T/qrels" "$T/results"
