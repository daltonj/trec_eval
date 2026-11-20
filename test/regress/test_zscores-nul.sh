# Refuse a NUL within a field or after a complete row, with its line number.
. test/regress/lib.sh
D=test/regress/zscores-nul

printf '1 map\000 0 1\n' > "$T/inside"
printf '1 map 0 1\n\000\n' > "$T/after"
line=1
for z in inside after; do
    printf "trec_eval.get_zscores: NUL character in line %s of zscores file '%s'\ntrec_eval: Quit in file '%s'\n" \
        "$line" "$T/$z" "$T/$z" > "$T/want"
    expect_output_status 2 /dev/null -Z "$T/$z" -m map "$D/qrels" "$D/results"
    expect_stderr 2 "$T/want" -Z "$T/$z" -m map "$D/qrels" "$D/results"
    line=2
done
