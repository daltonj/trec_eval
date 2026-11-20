# A float parameter whose count of documents does not fit in a long
# must not overflow an integer conversion. Rprec_mult must also retain
# the full cutoff as its denominator, which -Z makes visible.
# relstring.1e300 prints the whole ranking of d1, d2, d3: '102'.
. test/regress/lib.sh
D=test/regress/count-parameter-range
R=test/regress/results

# same MEASURE HUGE LARGE [ARGS...]: MEASURE.HUGE exits 0 and prints the
# values that MEASURE.LARGE prints.
same() {
    m=$1 huge=$2 large=$3
    shift 3
    save_output "$T/huge" -q "$@" -m $m.$huge $D/qrels $R
    save_output "$T/large" -q "$@" -m $m.$large $D/qrels $R
    [ "$(cut -f2- "$T/huge")" = "$(cut -f2- "$T/large")" ] ||
        fail "trec_eval -m $m.$huge differs from $m.$large"
}

# Recall levels outside [0, 1] and multiples of R that are not positive
# are refused, so only relstring and large multiples of R remain.
for m in relstring Rprec_mult; do
    same $m 1e300 1000000
done
same relstring -1e300 -1000000
same Rprec_mult_avgjg 1e300 1000000 -R qrels_jg

expect_output $D/out.relstring -q -m relstring.1e300 $D/qrels $R

for m in Rprec_mult Rprec_mult_avgjg; do
    format=qrels
    [ "$m" = Rprec_mult_avgjg ] && format=qrels_jg
    save_output "$T/out" -R "$format" -m "$m.1e300" $D/qrels $R
    awk '{print "1", $1, "0", "1e-19"}' "$T/out" > "$T/zscores"
    save_output "$T/out" -q -R "$format" -m "$m.1e300" -Z "$T/zscores" \
        $D/qrels $R
    awk '{print $2, $3}' "$T/out" > "$T/values"
    diff "$T/values" $D/out.huge-zscore || fail "$m: clamped denominator"
done
