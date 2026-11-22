# A UTF-8 byte-order mark, CRLF line ends and a CR ending the last line do
# not change what any reader reads. In the qrels copies the first line is a
# comment. A CR within a line is still refused.
. test/regress/lib.sh
Q=test/regress/qrels
P=test/regress/prefs
R=test/regress/results
Z=test/regress/zscores

# encode NAME FILE: write the copies $T/NAME.bom, .crlf and .lastcr of FILE.
encode() {
    { printf '\357\273\277'; cat "$2"; } > "$T/$1.bom"
    awk '{ printf "%s\r\n", $0 }' "$2" > "$T/$1.crlf"
    printf '%s\r' "$(cat "$2")" > "$T/$1.lastcr"
}
{ echo '# comment'; cat $Q; } > "$T/commented"
encode qrels "$T/commented"
encode prefs $P
encode results $R
encode zscores $Z

for format in qrels qrels_jg qrels_prefs; do
    save_output "$T/plain" -R $format $Q $R
    for ending in bom crlf lastcr; do
        expect_output "$T/plain" -R $format "$T/qrels.$ending" $R
    done
done
save_output "$T/plain" -R prefs $P $R
for ending in bom crlf lastcr; do
    expect_output "$T/plain" -R prefs "$T/prefs.$ending" $R
done
save_output "$T/plain" $Q $R
for ending in bom crlf lastcr; do
    expect_output "$T/plain" $Q "$T/results.$ending"
done
save_output "$T/plain" -m map -Z $Z $Q $R
for ending in bom crlf lastcr; do
    expect_output "$T/plain" -m map -Z "$T/zscores.$ending" $Q $R
done

printf '1 0 d1 1\n1 0 d2 0\r1 0 d3 1\n' > "$T/cr"
expect_message 2 "CR character without LF in line 2 of qrels file '$T/cr'" \
    "$T/cr" $R
