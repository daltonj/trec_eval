# Only the results path - means standard input. A results file whose name
# starts with - is read as a file, and a missing one is an error. The run
# in the file ranks a, b, c; the run on standard input ranks c, b, a.
. test/regress/lib.sh
D=$(pwd)/test/regress/results-path-dash
TREC_EVAL=$(cd "$(dirname "$TREC_EVAL")" && pwd)/$(basename "$TREC_EVAL")
cp "$D/run" "$T/-run"
cd "$T" || fail "cannot cd to $T"

expect_output "$D/out.run" -q -m relstring -- "$D/qrels" -run < "$D/run.stdin"
expect_message 2 "Cannot read results file '-x'" \
    -q -m relstring -- "$D/qrels" -x < "$D/run.stdin"
expect_output "$D/out.stdin" -q -m relstring "$D/qrels" - < "$D/run.stdin"
