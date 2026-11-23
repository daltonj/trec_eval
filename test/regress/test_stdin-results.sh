# Put the last record before, at and beyond the standard-input buffer
# growth (4094, then 8190 bytes), without a final newline. File and stdin
# must agree. Only an AddressSanitizer build catches a write past the buffer.
. test/regress/lib.sh
Q=test/regress/qrels.one
R=test/regress/results.one
for n in 4093 4094 4095 8190 8191; do
    pad=$(( n - $(wc -c < $R) - 1 ))
    {
        printf "#%${pad}s\n" ''
        printf '%s' "$(cat $R)"
    } > "$T/results"
    [ "$(wc -c < "$T/results")" -eq $n ] || fail "input is not $n bytes"
    save_output "$T/out" -q $Q "$T/results"
    expect_output "$T/out" -q $Q - < "$T/results"
done

# A directory on standard input makes fread fail.
expect_message 2 "Cannot read results file '-'" $Q - < test/regress
