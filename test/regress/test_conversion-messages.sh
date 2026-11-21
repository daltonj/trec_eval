# Checked conversions name their input. Numeric suffixes warn and retain
# the parsed prefix; unusable and non-finite parameters fail at init.
. test/regress/lib.sh
Q=test/regress/qrels
R=test/regress/results
for measure in iprec_at_recall.x iprec_at_recall..1,,.2 \
    11pt_avg.x ndcg.1=x; do
    expect_message 2 "could not be parsed" -m $measure $Q $R
done
for measure in iprec_at_recall.inf set_F.inf ndcg.1=inf; do
    expect_message 2 "is not finite" -m $measure $Q $R
done
expect_message 2 "could not be parsed (cutoff of measure 'P')." -m P.x $Q $R
expect_message 0 "partially read as \`1' (cutoff of measure 'P')." -m P.1.5 $Q $R
for format in qrels qrels_jg; do
    expect_message 0 "partially read as \`1' (document 'd1' of topic '1' in line 1 of qrels file 'test/regress/conversion-messages/qrels.partial')." \
        -R $format test/regress/conversion-messages/qrels.partial $R
done
while IFS='|' read -r measure value prefix context; do
    expect_message 0 "Warning, value \`$value' partially read as \`$prefix' ($context)." \
        -m "$measure" $Q $R
    [ "$value" != .25x ] || te_same output test/regress/conversion-messages/out.partial
done <<'EOF'
iprec_at_recall..25x|.25x|.25|cutoff of measure 'iprec_at_recall'
set_F.0.5x|0.5x|0.5|parameter of measure 'set_F'
ndcg.1=2x|2x|2|parameter '1' of measure 'ndcg'
EOF
