# Results read from standard input (-) are checked for a NUL byte as a
# results file is: exit 2 and a message naming the line. Input of any
# size is read whole: the reader's buffer starts at 4096 bytes, and
# inputs near that size give the same output as the file.
. test/regress/lib.sh

printf '1 Q0 d1 1 3 r\n1 Q0 d2\000x 2 2 r\n1 Q0 d3 3 1 r\n' > "$T/run.nul-field"
printf '1 Q0 d1 1 3 r\n\000\n1 Q0 d3 3 1 r\n' > "$T/run.nul-line"
for v in nul-field nul-line; do
    expect_message 2 "NUL character in line 2 of results file '-'" \
        -m num_ret test/regress/qrels - < "$T/run.$v"
done

expect_output test/out.test test/qrels.test - < test/results.test

# Inputs of n bytes: a padded comment, then 40 results for topic 301 that
# end at byte n with no final newline, so that the last result ends at or
# crosses byte 4094 or 8190, where the reader's buffer grows.
grep '^301 ' test/qrels.test > "$T/qrels.301"
head -n 40 test/results.test > "$T/head"
for n in 4093 4094 4095 8190; do
    pad=$(( n - $(wc -c < "$T/head") - 1 ))
    { printf "#%${pad}s\n" ''; printf '%s' "$(cat "$T/head")"; } > "$T/run.$n"
    [ "$(wc -c < "$T/run.$n")" -eq $n ] || fail "$T/run.$n is not $n bytes"
    save_output "$T/out.$n" -q "$T/qrels.301" "$T/run.$n"
    expect_output "$T/out.$n" -q "$T/qrels.301" - < "$T/run.$n"
done
