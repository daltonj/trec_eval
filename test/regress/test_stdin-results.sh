# Input of any size is read whole from standard input (-): the reader's
# buffer starts at 4096 bytes, and inputs near that size give the same
# output as the file.
. test/regress/lib.sh

expect_output test/out.test test/qrels.test - < test/results.test

# Inputs of n bytes: a padded comment, then 40 results for topic 301 that
# end at byte n with no final newline, so that the last result ends at or
# crosses byte 4094 or 8190, where the reader's buffer grows.
grep '^301 ' test/qrels.test > "$T/qrels.301"
head -n 40 test/results.test > "$T/head"
for n in 4093 4094 4095 8190; do
    pad=$(( n - $(wc -c < "$T/head") - 1 ))
    { printf "#%${pad}s\n" ''; printf '%s' "$(cat "$T/head")"; } > "$T/results.$n"
    [ "$(wc -c < "$T/results.$n")" -eq $n ] || fail "$T/results.$n is not $n bytes"
    save_output "$T/out.$n" -q "$T/qrels.301" "$T/results.$n"
    expect_output "$T/out.$n" -q "$T/qrels.301" - < "$T/results.$n"
done
