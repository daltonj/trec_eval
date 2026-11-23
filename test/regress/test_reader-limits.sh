# Check file allocation sizes, stdin allocation failures and zscore search
# bounds with small inputs and substituted allocation/position functions.
. test/regress/lib.sh

./test_reader-limits file-sizes test/regress/qrels 2> "$T/error" ||
    fail "a reader did not ask for a buffer of the whole file"

printf "trec_eval.get_results: Cannot copy results file '-'\n" > "$T/err"
./test_reader-limits stdin-malloc test/regress/results 2> "$T/error" ||
    fail "stdin reader did not return UNDEF after malloc failed"
diff "$T/err" "$T/error" || fail "wrong stdin malloc failure message"

# Filling the first 4094 bytes forces growth of the 4096-byte buffer.
printf '%4094s' '' > "$T/growing"
./test_reader-limits stdin-realloc "$T/growing" 2> "$T/error" ||
    fail "stdin reader did not return UNDEF after realloc failed"
diff "$T/err" "$T/error" || fail "wrong stdin realloc failure message"

./test_reader-limits zscores || fail "zscores search exceeded its count"
