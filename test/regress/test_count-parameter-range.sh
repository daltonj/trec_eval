# Clamp relstring's length before conversion to long. Both Rprec measures
# retain an enormous cutoff as the denominator, rather than clamping it.
. test/regress/lib.sh
Q=test/regress/qrels.one
R=test/regress/results.one

# Converting 1e300 to long is undefined: x86-64 gives an empty string, and
# an UndefinedBehaviorSanitizer build reports it on stderr.
expect_result 0 test/regress/count-parameter-range/out.relstring /dev/null \
    -q -m relstring.1e300 $Q $R

# With one relevant doc, Z = (1/1e19)/1e-19 = 1. A cutoff clamped to
# LONG_MAX would change the value: to 1.0842 where long has 64 bits.
while read -r measure format; do
    row=${measure}_10000000000000000000.00
    printf '1 %s 0 1e-19\n' "$row" > "$T/zscores"
    printf 'Z%s\tall\t1.0000\n' "$row" > "$T/want"
    expect_output "$T/want" -R $format -m $measure.1e19 -Z "$T/zscores" $Q $R
done <<'EOF'
Rprec_mult qrels
Rprec_mult_avgjg qrels_jg
EOF
