# Lines may end with LF or CRLF. A CR that is not followed by LF is refused
# unless it ends the file, where the reader appends LF. The message names
# the line and the file.
. test/regress/lib.sh
Q=test/regress/qrels
P=test/regress/prefs
R=test/regress/results

printf '1 map 0.5 0.25\n2 map 0.5 0.25\n' > "$T/zscores"
for f in $Q $P $R "$T/zscores"; do
    awk '{ printf "%s\r\n", $0 }' $f > "$T/${f##*/}.crlf"
    printf '%s\r' "$(cat $f)" > "$T/${f##*/}.lastcr"
    tr '\n' '\r' < $f > "$T/${f##*/}.cr"
done
printf '1 0 d1 1\n1 0 d2 0\r1 0 d3 1\n' > "$T/qrels.mixed"

# CRLF files, and files whose last line ends in CR, give the same values
# as the LF files.
m="-q -m num_ret -m map"
save_output "$T/lf" $m $Q $R
for e in crlf lastcr; do
    expect_output "$T/lf" $m "$T/qrels.$e" "$T/results.$e"
    expect_output "$T/lf" $m "$T/qrels.$e" - < "$T/results.$e"
done
m="-q -R qrels_jg -m map_avgjg"
save_output "$T/lf" $m $Q $R
for e in crlf lastcr; do
    expect_output "$T/lf" $m "$T/qrels.$e" "$T/results.$e"
done
m="-q -R qrels_prefs -m prefs_num_prefs_poss"
save_output "$T/lf" $m $Q $R
for e in crlf lastcr; do
    expect_output "$T/lf" $m "$T/qrels.$e" "$T/results.$e"
done
m="-q -R prefs -m prefs_num_prefs_poss"
save_output "$T/lf" $m $P $R
for e in crlf lastcr; do
    expect_output "$T/lf" $m "$T/prefs.$e" $R
done
m="-q -m map"
save_output "$T/lf" $m -Z "$T/zscores" $Q $R
for e in crlf lastcr; do
    expect_output "$T/lf" $m -Z "$T/zscores.$e" $Q $R
done

cr="CR character without LF in line"
expect_message 2 "$cr 1 of results file '$T/results.cr'" \
    -m num_ret $Q "$T/results.cr"
expect_message 2 "$cr 1 of results file '-'" \
    -m num_ret $Q - < "$T/results.cr"
expect_message 2 "$cr 1 of qrels file '$T/qrels.cr'" \
    -m num_rel "$T/qrels.cr" $R
expect_message 2 "$cr 1 of qrels file '$T/qrels.cr'" \
    -R qrels_jg -m map_avgjg "$T/qrels.cr" $R
expect_message 2 "$cr 1 of qrels file '$T/qrels.cr'" \
    -R qrels_prefs -m prefs_num_prefs_poss "$T/qrels.cr" $R
expect_message 2 "$cr 1 of prefs file '$T/prefs.cr'" \
    -R prefs -m prefs_num_prefs_poss "$T/prefs.cr" $R
expect_message 2 "$cr 1 of zscores file '$T/zscores.cr'" \
    -Z "$T/zscores.cr" -m map $Q $R
expect_message 2 "$cr 2 of qrels file '$T/qrels.mixed'" \
    -m num_rel "$T/qrels.mixed" $R
