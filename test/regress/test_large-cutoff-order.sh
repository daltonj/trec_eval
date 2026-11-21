# Cutoffs that differ by 2^31 or more are sorted in increasing order. The
# run ranks a (grade 1), b (grade 0) and c (grade 1), so P_1 = 1,
# recall_1 = recall_2 = 1/2, and every cutoff above 3 has P = 2 / cutoff
# (0.0000 at four decimals) and recall 1.
. test/regress/lib.sh
D=test/regress/large-cutoff-order
Q="$T/qrels"
R="$T/results"
printf '1 0 a 1\n1 0 b 0\n1 0 c 1\n' > "$Q"
printf '1 Q0 a 1 3 r\n1 Q0 b 2 2 r\n1 Q0 c 3 1 r\n' > "$R"

expect_output $D/out.P.portable -m P.2147483647,1 "$Q" "$R"
expect_output $D/out.recall.portable -m recall.2,2147483647,1 "$Q" "$R"

if "$TREC_EVAL" -m P.4294967297,1 "$Q" "$R" >"$T/out" 2>"$T/err"; then
    diff $D/out.P "$T/out" || fail "wrong precision at large cutoffs"
elif grep -F "Error: value \`4294967297' out of range" "$T/err" >/dev/null; then
    echo "SKIP $0: cutoffs above 2^32 need a 64-bit long" >&2
    exit 0
else
    cat "$T/err" >&2
    fail "large cutoff evaluation failed"
fi

expect_output $D/out.recall -m recall.2,4294967297,1 "$Q" "$R"
