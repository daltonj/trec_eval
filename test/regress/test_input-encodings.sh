# CRLF, a final CR and an initial UTF-8 byte-order mark preserve the
# values read from each input format. A CR within a line is refused.
. test/regress/lib.sh
Q=test/regress/qrels
P=test/regress/prefs
R=test/regress/results
Z=test/regress/zscores

for f in $Q $P $R $Z; do
    awk '{ printf "%s\r\n", $0 }' $f > "$T/${f##*/}.crlf"
    printf '%s\r' "$(cat $f)" > "$T/${f##*/}.lastcr"
    { printf '\357\273\277'; cat $f; } > "$T/${f##*/}.bom"
    tr '\n' '\r' < $f > "$T/${f##*/}.cr"
done
printf '1 0 d1 1\n1 0 d2 0\r1 0 d3 1\n' > "$T/qrels.mixed"
{ printf '\357\273\277# comment\n'; cat $Q; } > "$T/qrels.comment"

m="-c -q -m num_q -m num_ret -m num_rel -m map"
save_output "$T/qrels" $m $Q $R
expect_output "$T/qrels" $m "$T/qrels.comment" $R
save_output "$T/qrels_jg" -q -R qrels_jg -m map_avgjg $Q $R
save_output "$T/qrels_prefs" -q -R qrels_prefs -m prefs_num_prefs_poss $Q $R
save_output "$T/prefs" -q -R prefs -m prefs_num_prefs_poss $P $R
save_output "$T/zscores" -q -m map -Z $Z $Q $R
for e in crlf lastcr bom; do
    expect_output "$T/qrels" $m "$T/qrels.$e" $R
    expect_output "$T/qrels" $m $Q "$T/results.$e"
    expect_output "$T/qrels" $m $Q - < "$T/results.$e"
    expect_output "$T/qrels_jg" -q -R qrels_jg -m map_avgjg "$T/qrels.$e" $R
    expect_output "$T/qrels_prefs" -q -R qrels_prefs -m prefs_num_prefs_poss "$T/qrels.$e" $R
    expect_output "$T/prefs" -q -R prefs -m prefs_num_prefs_poss "$T/prefs.$e" $R
    expect_output "$T/zscores" -q -m map -Z "$T/zscores.$e" $Q $R
done

cr="CR character without LF in line"
expect_message 2 "$cr 1 of results file '$T/results.cr'" \
    -m num_ret "$Q" "$T/results.cr"
expect_message 2 "$cr 1 of results file '-'" \
    -m num_ret "$Q" - < "$T/results.cr"
expect_message 2 "$cr 1 of qrels file '$T/qrels.cr'" \
    -m num_rel "$T/qrels.cr" "$R"
expect_message 2 "$cr 1 of qrels file '$T/qrels.cr'" \
    -R qrels_jg -m map_avgjg "$T/qrels.cr" "$R"
expect_message 2 "$cr 1 of qrels file '$T/qrels.cr'" \
    -R qrels_prefs -m prefs_num_prefs_poss "$T/qrels.cr" "$R"
expect_message 2 "$cr 1 of prefs file '$T/prefs.cr'" \
    -R prefs -m prefs_num_prefs_poss "$T/prefs.cr" "$R"
expect_message 2 "$cr 1 of zscores file '$T/zscores.cr'" \
    -Z "$T/zscores.cr" -m map "$Q" "$R"
expect_message 2 "$cr 2 of qrels file '$T/qrels.mixed'" \
    -m num_rel "$T/qrels.mixed" "$R"
