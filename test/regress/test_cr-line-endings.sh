# Lines may end with LF or CRLF. A CR that is not followed by LF is refused
# unless it ends the file, where the reader appends LF. The message names
# the line and the file. Before, a CR-only results file was read as one
# line and evaluated without a message. The CRLF and CR files are written
# here from the LF fixtures, so that the fixtures stay text.
. test/regress/lib.sh
D=test/regress/cr-line-endings

for f in qrels prefs results zscores; do
    awk '{ printf "%s\r\n", $0 }' $D/$f > "$T/$f.crlf"
    tr '\n' '\r' < $D/$f > "$T/$f.cr"
done
printf '1 0 d1 1\n1 0 d2 0\r1 0 d3 1\n' > "$T/qrels.mixed"

# CRLF files give the same values as the LF files.
for e in "" .crlf; do
    q=$D/qrels r=$D/results
    [ -n "$e" ] && q=$T/qrels$e r=$T/results$e
    expect_output $D/out.qrels -q -m num_ret -m map "$q" "$r"
    expect_output $D/out.qrels -q -m num_ret -m map "$q" - < "$r"
    expect_output $D/out.qrels_jg -q -R qrels_jg -m map_avgjg "$q" "$r"
    expect_output $D/out.qrels_prefs -q -R qrels_prefs \
        -m prefs_num_prefs_poss "$q" "$r"
done
expect_output $D/out.prefs -q -R prefs -m prefs_num_prefs_poss \
    "$T/prefs.crlf" $D/results
expect_output $D/out.zscores -q -Z "$T/zscores.crlf" -m map $D/qrels $D/results

cr="CR character without LF in line"
expect_message 2 "$cr 1 of results file '$T/results.cr'" \
    -m num_ret $D/qrels "$T/results.cr"
expect_message 2 "$cr 1 of results file '-'" \
    -m num_ret $D/qrels - < "$T/results.cr"
expect_message 2 "$cr 1 of qrels file '$T/qrels.cr'" \
    -m num_rel "$T/qrels.cr" $D/results
expect_message 2 "$cr 1 of qrels file '$T/qrels.cr'" \
    -R qrels_jg -m map_avgjg "$T/qrels.cr" $D/results
expect_message 2 "$cr 1 of qrels file '$T/qrels.cr'" \
    -R qrels_prefs -m prefs_num_prefs_poss "$T/qrels.cr" $D/results
expect_message 2 "$cr 1 of prefs file '$T/prefs.cr'" \
    -R prefs -m prefs_num_prefs_poss "$T/prefs.cr" $D/results
expect_message 2 "$cr 1 of zscores file '$T/zscores.cr'" \
    -Z "$T/zscores.cr" -m map $D/qrels $D/results
expect_message 2 "$cr 2 of qrels file '$T/qrels.mixed'" \
    -m num_rel "$T/qrels.mixed" $D/results
