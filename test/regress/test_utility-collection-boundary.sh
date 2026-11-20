# -N LONG_MAX must not overflow when computing utility's fourth cell.
# Run with UBSan to detect intermediate signed overflow as well as output.
. test/regress/lib.sh
D=test/regress/utility-collection-boundary
expect_output $D/out.portable -N 2147483647 -m utility.0,0,0,1 $D/qrels $D/run

T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT
if "$TREC_EVAL" -N 9223372036854775807 -m utility.0,0,0,1 $D/qrels $D/run >"$T/out" 2>"$T/err"; then
    diff $D/out.max "$T/out" || fail "wrong utility at the collection boundary"
elif grep -F "Error: value \`9223372036854775807' out of range" "$T/err" >/dev/null; then
    echo "SKIP $0: this collection size needs a 64-bit long" >&2
else
    cat "$T/err" >&2
    fail "large collection evaluation failed"
fi
exit 0
