# A float parameter whose count of documents does not fit in a long
# must not overflow an integer conversion. Rprec_mult must also retain
# the full cutoff as its denominator, which -Z makes visible.
# relstring.1e300 prints the whole ranking of d1, d2, d3: '102'.
. test/regress/lib.sh
D=test/regress/count-parameter-range
Q="$T/qrels"
R=test/regress/results
printf '1 0 d1 1\n1 0 d2 0\n1 0 d3 2\n' > "$Q"

expect_output $D/out.relstring -q -m relstring.1e300 "$Q" $R

# A stddev of 1e-19 multiplies the score by 1e19. A denominator
# clamped to a 64-bit LONG_MAX would then print about 2.17 instead of 0.
for measure in Rprec_mult Rprec_mult_avgjg; do
    format=qrels
    [ "$measure" = Rprec_mult_avgjg ] && format=qrels_jg
    save_output "$T/out" -R "$format" -m "$measure.1e300" "$Q" $R
    awk '{print "1", $1, "0", "1e-19"}' "$T/out" > "$T/zscores"
    save_output "$T/out" -q -R "$format" -m "$measure.1e300" -Z "$T/zscores" \
        "$Q" $R
    awk '{print $2, $3}' "$T/out" > "$T/values"
    diff "$T/values" $D/out.huge-zscore || fail "$measure: clamped denominator"
done
