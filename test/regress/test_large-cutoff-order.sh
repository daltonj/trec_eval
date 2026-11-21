# Integer cutoffs differing by more than INT_MAX still sort correctly.
# The run has one relevant doc, so precision at a huge cutoff rounds to 0.
. test/regress/lib.sh
D=test/regress/large-cutoff-order
Q=test/regress/rbp-gain-range/qrels.one
R=test/regress/rbp-gain-range/results.one

expect_output $D/out.P.portable -m P.2147483647,1 "$Q" "$R"

if "$TREC_EVAL" -m P.4294967297,1 "$Q" "$R" >"$T/out" 2>"$T/err"; then
    diff $D/out.P "$T/out" || fail "wrong precision at large cutoffs"
elif grep -F "Error: value \`4294967297' out of range" "$T/err" >/dev/null; then
    echo "SKIP $0: cutoffs above 2^32 need a 64-bit long" >&2
    exit 0
else
    cat "$T/err" >&2
    fail "large cutoff evaluation failed"
fi
