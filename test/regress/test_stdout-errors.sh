# A failed write to standard output exits 8 with a message, instead of 0
# with the results lost.
. test/regress/lib.sh
"$TREC_EVAL" test/regress/qrels.one test/regress/results.one \
    >&- 2> "$T/error"
status=$?
[ $status = 8 ] || fail "closed standard output: exit $status, expected 8"
grep -q "Can't write to standard output" "$T/error" ||
    fail "no message for a failed write to standard output"
