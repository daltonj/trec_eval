# -N LONG_MAX must not overflow when computing utility's fourth cell.
. test/regress/lib.sh
D=test/regress/utility-collection-boundary
Q=$T/qrels
R=$T/run
printf '1 0 a 1\n' > "$Q"
printf '1 Q0 a 1 1 r\n' > "$R"
expect_result 0 $D/out.portable /dev/null -N 2147483647 -m utility.0,0,0,1 "$Q" "$R"

if "$TREC_EVAL" -N 9223372036854775807 -m utility.0,0,0,1 "$Q" "$R" \
        > "$T/out" 2> "$T/err"; then
    diff $D/out.max "$T/out" || fail "wrong utility at the collection boundary"
    [ ! -s "$T/err" ] || {
        cat "$T/err" >&2
        fail "message at the collection boundary"
    }
elif grep -F "Error: value \`9223372036854775807' out of range" "$T/err" >/dev/null; then
    echo "SKIP $0: this collection size needs a 64-bit long" >&2
else
    cat "$T/err" >&2
    fail "large collection evaluation failed"
fi
