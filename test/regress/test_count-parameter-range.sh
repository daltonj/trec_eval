# A float parameter whose count of documents does not fit in a long
# must not overflow an integer conversion. Rprec_mult must also retain
# the full cutoff as its denominator, which -Z makes visible.
# relstring.1e300 prints the whole ranking of d1, d2, d3: '102'.
. test/regress/lib.sh
D=test/regress/count-parameter-range

# same MEASURE HUGE LARGE [ARGS...]: MEASURE.HUGE exits 0 without a
# runtime error and prints the values that MEASURE.LARGE prints.
same() {
    m=$1 huge=$2 large=$3
    shift 3
    expect_status 0 -q "$@" -m $m.$huge $D/qrels $D/run
    "$TREC_EVAL" -q "$@" -m $m.$huge $D/qrels $D/run 2>&1 >/dev/null |
        grep -q "runtime error" && fail "trec_eval -m $m.$huge: runtime error"
    a=$("$TREC_EVAL" -q "$@" -m $m.$huge $D/qrels $D/run | cut -f2-)
    b=$("$TREC_EVAL" -q "$@" -m $m.$large $D/qrels $D/run | cut -f2-)
    [ "$a" = "$b" ] || fail "trec_eval -m $m.$huge differs from $m.$large"
}

# Recall levels outside [0, 1] and multiples of R that are not positive
# are refused, so only relstring and large multiples of R remain.
for m in relstring Rprec_mult; do
    same $m 1e300 1000000
done
same relstring -1e300 -1000000
same Rprec_mult_avgjg 1e300 1000000 -R qrels_jg

expect_output $D/out.relstring -q -m relstring.1e300 $D/qrels $D/run

tmp=$(mktemp -d "${TMPDIR:-/tmp}/count-parameter-range.XXXXXX") || exit 1
trap 'rm -rf "$tmp"' EXIT HUP INT TERM
for m in Rprec_mult Rprec_mult_avgjg; do
    format=qrels
    [ "$m" = Rprec_mult_avgjg ] && format=qrels_jg
    "$TREC_EVAL" -R "$format" -m "$m.1e300" $D/qrels $D/run > "$tmp/out" ||
        fail "$m: cannot get row name"
    awk '{print "1", $1, "0", "1e-19"}' "$tmp/out" > "$tmp/zscores"
    "$TREC_EVAL" -q -R "$format" -m "$m.1e300" -Z "$tmp/zscores" \
        $D/qrels $D/run > "$tmp/out" || fail "$m: cannot compute zscore"
    awk '{print $2, $3}' "$tmp/out" > "$tmp/values"
    diff "$tmp/values" $D/out.huge-zscore || fail "$m: clamped denominator"
done
exit 0
