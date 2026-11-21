# A judgments, results or zscores file that contains a NUL byte is
# refused with exit 2 and a message naming the file and the line of the
# first NUL, counting every line of the file.
. test/regress/lib.sh
Q=test/regress/qrels
R=test/regress/results

printf '1 0 d1 1\n\000\n1 0 d2 1\n' > "$T/qrels.2"
printf '1 0 d1\000x 1\n' > "$T/qrels.1"
printf '# c\n1 0 d1 1\n1 0 d2\000 1\n' > "$T/qrels.3"
printf '1 A 0 d1 1\n\000\n1 A 0 d2 1\n' > "$T/prefs.2"
printf '1 A 0 d1\000x 1\n' > "$T/prefs.1"
printf '1 Q0 d1 1 3 r\n\000\n1 Q0 d3 3 1 r\n' > "$T/results.2"
printf '1 Q0 d1\000x 1 3 r\n' > "$T/results.1"
printf '1 map 0 1\n\000\n' > "$T/zscores.2"
printf '1 map\000 0 1\n' > "$T/zscores.1"

for n in 1 2 3; do
    message="NUL character in line $n of qrels file '$T/qrels.$n'"
    expect_message 2 "get_qrels: $message" -m num_rel "$T/qrels.$n" $R
    expect_message 2 "get_qrels_jg: $message" \
        -R qrels_jg -m map_avgjg "$T/qrels.$n" $R
    expect_message 2 "get_qrels_prefs: $message" \
        -R qrels_prefs -m prefs_num_prefs_poss "$T/qrels.$n" $R
done
for n in 1 2; do
    expect_message 2 "get_prefs: NUL character in line $n of prefs file '$T/prefs.$n'" \
        -R prefs -m prefs_num_prefs_poss "$T/prefs.$n" $R
    expect_message 2 "get_results: NUL character in line $n of results file '$T/results.$n'" \
        -m num_ret $Q "$T/results.$n"
    expect_message 2 "get_results: NUL character in line $n of results file '-'" \
        -m num_ret $Q - < "$T/results.$n"
    expect_message 2 "get_zscores: NUL character in line $n of zscores file '$T/zscores.$n'" \
        -Z "$T/zscores.$n" -m map $Q $R
done
