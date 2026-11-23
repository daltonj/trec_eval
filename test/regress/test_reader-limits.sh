# Check stdin allocation failures, zscore search bounds, the read limit and
# file allocation sizes with small inputs, a lowered limit and substituted
# allocation, read and position functions.
. test/regress/lib.sh

printf "trec_eval.get_results: Cannot copy results file '-'\n" > "$T/want"
./test_reader-limits stdin-malloc < /dev/null 2> "$T/got" ||
    fail "stdin reader mishandled a failed malloc"
diff "$T/want" "$T/got" || fail "wrong stdin malloc failure message"

# Filling the first 4094 bytes forces growth of the 4096-byte buffer.
printf '%4094s' '' > "$T/growing"
./test_reader-limits stdin-realloc < "$T/growing" 2> "$T/got" ||
    fail "stdin reader mishandled a failed realloc"
diff "$T/want" "$T/got" || fail "wrong stdin realloc failure message"

./test_reader-limits zscores || fail "zscores search exceeded its count"

printf abcde | ./test_reader-limits read-chunks ||
    fail "te_fread read more than TE_READ_CHUNK bytes at a time"

# The size UINT_MAX + 1 needs a long wider than unsigned.
need_long64
./test_reader-limits file-size 2> /dev/null ||
    fail "te_read_lines did not ask for a buffer of the whole file"
