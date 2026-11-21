# Measure-specific ranges are checked at initialization.
. test/regress/lib.sh
Q=test/qrels.test
R=test/results.test
while IFS='|' read -r measure message; do
    expect_message 2 "$message" -m "$measure" $Q $R
done <<'EOF'
iprec_at_recall.-1,.5|recall level -1 of measure 'iprec_at_recall' is not in [0, 1]
iprec_at_recall..5,1.5|recall level 1.5 of measure 'iprec_at_recall' is not in [0, 1]
11pt_avg..5,1.5|recall level 1.5 of measure '11pt_avg' is not in [0, 1]
Rprec_mult.0,1|multiple 0 of measure 'Rprec_mult' is not positive
Rprec_mult.-1|multiple -1 of measure 'Rprec_mult' is not positive
Rprec_mult_avgjg.0|multiple 0 of measure 'Rprec_mult_avgjg' is not positive
set_F.-1|weight -1 of measure 'set_F' is negative
rbp.p=1|persistence p=1 of measure 'rbp' is not in [0, 1)
rbp.p=-0.5|persistence p=-0.5 of measure 'rbp' is not in [0, 1)
rbp_resid.p=1.5|persistence p=1.5 of measure 'rbp_resid' is not in [0, 1)
EOF
# Inclusive endpoints, repeated 11pt_avg levels and small positive values.
for measure in iprec_at_recall.0,1 11pt_avg.1,0,.5,.5 \
    Rprec_mult.0.01 set_F.0 rbp.p=0 rbp_resid.p=0.99; do
    expect_status 0 -m $measure $Q $R
done
