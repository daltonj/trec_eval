# These buffer-boundary cases cover the rewritten standard-input reader;
# they also pass on upstream main. Put the last record before, at and
# beyond buffer growth (4094, then 8190 bytes), without a final newline.
# File and stdin must agree.
. test/regress/lib.sh
printf '1 0 d 1\n' > "$T/qrels"
printf '1 Q0 d 1 1 r\n' > "$T/head"
for n in 4093 4094 4095 8190 8191; do
    pad=$(( n - $(wc -c < "$T/head") - 1 ))
    {
        printf "#%${pad}s\n" ''
        printf '%s' "$(cat "$T/head")"
    } > "$T/results"
    [ "$(wc -c < "$T/results")" -eq $n ] || fail "input is not $n bytes"
    save_output "$T/out" -q "$T/qrels" "$T/results"
    expect_output "$T/out" -q "$T/qrels" - < "$T/results"
done

expect_message 2 "Cannot read results file '-'" "$T/qrels" - < test/regress
