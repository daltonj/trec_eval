# Zscores need finite means and finite, non-negative standard deviations.
. test/regress/lib.sh
D=test/regress/zscores-numbers
Q="$T/qrels"
R="$T/results"
printf '1 0 d 1\n' > "$Q"
printf '1 Q0 d 1 1 r\n' > "$R"

for field in mean stddev; do
    values='abc 1x nan inf -inf 1e999'
    [ "$field" = mean ] || values="$values -1"
    for value in $values; do
        if [ "$field" = mean ]; then
            printf '1 map 0 1\n1 map %s 1\n' "$value" > "$T/zscores"
        else
            printf '1 map 0 1\n1 map 0 %s\n' "$value" > "$T/zscores"
        fi
        loc="line 2 of zscores file '$T/zscores'"
        unusable="trec_eval.get_zscores: Unusable $field '$value' in $loc"
        case $value in
        abc) want="Error: value \`abc' could not be parsed ($field in $loc)." ;;
        nan) want="Error: value \`nan' is not a number ($field in $loc)." ;;
        1e999) want="Warning, value \`1e999' out of range, read as \`inf' ($field in $loc).
$unusable" ;;
        *) want=$unusable ;;
        esac
        printf "%s\ntrec_eval: Quit in file '%s'\n" "$want" "$T/zscores" \
            > "$T/want"
        expect_result 2 /dev/null "$T/want" -Z "$T/zscores" -m map "$Q" "$R"
    done
done

printf '1 map 1 1e-310\n' > "$T/zscores"
expect_status 0 -Z "$T/zscores" -m map "$Q" "$R"
printf '1 map -1 2\n' > "$T/zscores"
expect_output $D/out.one -q -Z "$T/zscores" -m map "$Q" "$R"
printf '1 map 1 0\n' > "$T/zscores"
expect_output $D/out.zero -q -Z "$T/zscores" -m map "$Q" "$R"
printf '1 map 0 0\n' > "$T/zscores"
expect_output $D/out.missing -q -Z "$T/zscores" -m map "$Q" "$R"
