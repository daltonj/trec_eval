# Clamp relstring's length before conversion to long. Both Rprec measures
# retain an enormous cutoff as the denominator, rather than clamping it.
. test/regress/lib.sh
D=test/regress/count-parameter-range
Q=test/regress/rbp-gain-range/qrels.one
R=test/regress/rbp-gain-range/results.one
expect_output $D/out.relstring -q -m relstring.1e300 $Q $R

# A stddev of 1e-19 multiplies the score by 1e19. Clamping to 64-bit
# LONG_MAX would give about 1.08 instead of 0 for the one relevant doc.
for measure in Rprec_mult Rprec_mult_avgjg; do
    format=qrels
    [ "$measure" = Rprec_mult_avgjg ] && format=qrels_jg
    save_output "$T/out" -R "$format" -m "$measure.1e300" $Q $R
    awk '{print "t", $1, "0", "1e-19"}' "$T/out" > "$T/zscores"
    save_output "$T/out" -q -R "$format" -m "$measure.1e300" -Z "$T/zscores" $Q $R
    awk '{print $2, $3}' "$T/out" > "$T/values"
    diff "$T/values" $D/out.huge-zscore || fail "$measure: clamped denominator"
done
