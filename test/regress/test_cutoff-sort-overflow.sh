# Cutoffs that differ by 2^31 or more must still be sorted in increasing
# order. The fixture ranks a (grade 1), b (grade 0) and c (grade 1), so
# P_1 = 1, recall_1 = recall_2 = 1/2, and every cutoff above 3 has
# P = 2 / cutoff (0.0000 at four decimals) and recall 1.
. test/regress/lib.sh
d=test/regress/cutoff-sort-overflow

expect_output $d/out.P.portable -m P.2147483647,1 $d/qrels $d/results
expect_output $d/out.recall.portable -m recall.2,2147483647,1 $d/qrels $d/results

T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT
if "$TREC_EVAL" -m P.4294967297,1 $d/qrels $d/results >"$T/out" 2>"$T/err"; then
    diff $d/out.P "$T/out" || fail "wrong precision at large cutoffs"
elif grep -F "Error: value \`4294967297' out of range" "$T/err" >/dev/null; then
    echo "SKIP $0: cutoffs above 2^32 need a 64-bit long" >&2
    exit 0
else
    cat "$T/err" >&2
    fail "large cutoff evaluation failed"
fi

expect_output $d/out.recall -m recall.2,4294967297,1 $d/qrels $d/results
