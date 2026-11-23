# Clamp relstring's length before conversion to long. Both Rprec measures
# retain an enormous cutoff as the denominator, rather than clamping it.
. test/regress/lib.sh
D=test/regress/count-parameter-range
Q="$T/qrels"
R="$T/results"
printf 't 0 a 1\n' > "$Q"
printf 't Q0 a 1 1 r\n' > "$R"

# Converting 1e300 to long is undefined: x86-64 gives an empty string, and
# an UndefinedBehaviorSanitizer build reports it on stderr.
expect_result 0 $D/out.relstring /dev/null -q -m relstring.1e300 "$Q" "$R"

# For one relevant doc, dividing by stddev 1e-19 makes the correct 1e-300
# precision print as 0.0000, but a 64-bit LONG_MAX clamp as about 1.08.
for measure in Rprec_mult Rprec_mult_avgjg; do
    format=qrels
    [ "$measure" = Rprec_mult_avgjg ] && format=qrels_jg
    save_output "$T/out" -R "$format" -m "$measure.1e300" "$Q" "$R"
    awk '{print "t", $1, "0", "1e-19"}' "$T/out" > "$T/zscores"
    save_output "$T/out" -q -R "$format" -m "$measure.1e300" \
        -Z "$T/zscores" "$Q" "$R"
    awk '{print $2, $3}' "$T/out" > "$T/values"
    diff "$T/values" $D/out.huge-zscore || fail "$measure: clamped denominator"
done
