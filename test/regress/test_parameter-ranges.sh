# Measure parameters outside the range in which the measure is defined
# are refused when the measure is initialized, with exit status 2: recall
# levels outside [0, 1], multiples of R that are not positive, a negative
# set_F weight and an rbp persistence p outside [0, 1).
. test/regress/lib.sh
Q=test/qrels.test
R=test/results.test

expect_message 2 "recall level -1 of measure 'iprec_at_recall' is not in [0, 1]" \
    -m iprec_at_recall.-1,.5 $Q $R
expect_message 2 "recall level 1.5 of measure 'iprec_at_recall' is not in [0, 1]" \
    -m iprec_at_recall..5,1.5 $Q $R
expect_message 2 "recall level -0.5 of measure '11pt_avg' is not in [0, 1]" \
    -m 11pt_avg.-0.5,.5 $Q $R
expect_message 2 "recall level 1.5 of measure '11pt_avg' is not in [0, 1]" \
    -m 11pt_avg..5,1.5 $Q $R
expect_message 2 "multiple 0 of measure 'Rprec_mult' is not positive" \
    -m Rprec_mult.0,1 $Q $R
expect_message 2 "multiple -1 of measure 'Rprec_mult' is not positive" \
    -m Rprec_mult.-1 $Q $R
expect_message 2 "multiple 0 of measure 'Rprec_mult_avgjg' is not positive" \
    -m Rprec_mult_avgjg.0 $Q $R
expect_message 2 "weight -1 of measure 'set_F' is negative" \
    -m set_F.-1 $Q $R
expect_message 2 "persistence p=1 of measure 'rbp' is not in [0, 1)" \
    -m rbp.p=1 $Q $R
expect_message 2 "persistence p=-0.5 of measure 'rbp' is not in [0, 1)" \
    -m rbp.p=-0.5 $Q $R
expect_message 2 "persistence p=1.5 of measure 'rbp_resid' is not in [0, 1)" \
    -m rbp_resid.p=1.5 $Q $R

# The bounds that are part of each range, and the defaults, are accepted.
# 11pt_avg may repeat a recall level, in any order.
for measure in iprec_at_recall.0,1 iprec_at_recall 11pt_avg.1,0 11pt_avg \
         11pt_avg.0.5,0.5 \
         Rprec_mult.0.01 Rprec_mult set_F.0 set_F rbp.p=0 rbp \
         rbp_resid.p=0.99 rbp_resid; do
    expect_status 0 -m $measure $Q $R
done
