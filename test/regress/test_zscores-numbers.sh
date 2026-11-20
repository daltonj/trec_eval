# Zscores need finite means and finite, non-negative standard deviations.
. test/regress/lib.sh
D=test/regress/zscores-numbers

for field in mean stddev; do
    values='abc nan inf -inf 1e999'
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
        abc) want="Error: value \`abc' could not be parsed ($loc)." ;;
        nan) want="Error: value \`nan' is not a number ($loc)." ;;
        1e999) want="Warning, value \`1e999' out of range, read as \`inf' ($loc).
$unusable" ;;
        *) want=$unusable ;;
        esac
        printf "%s\ntrec_eval: Quit in file '%s'\n" "$want" "$T/zscores" \
            > "$T/want"
        expect_output_status 2 /dev/null \
            -Z "$T/zscores" -m map "$D/qrels" "$D/results"
        expect_stderr 2 "$T/want" \
            -Z "$T/zscores" -m map "$D/qrels" "$D/results"
    done
done

for row in '1 map -1 2' '1 map 1 0' '1 map 0 0' '1 map 1 1e-310' \
           '1 map 1x 1' '1 map 0 1x'; do
    printf '%s\n' "$row" > "$T/zscores"
    expect_status 0 -Z "$T/zscores" -m map "$D/qrels" "$D/results"
done
printf '1 map -1 2\n' > "$T/zscores"
expect_output "$D/out.one" -q -Z "$T/zscores" -m map "$D/qrels" "$D/results"
printf '1 map 1 0\n' > "$T/zscores"
expect_output "$D/out.zero" -q -Z "$T/zscores" -m map "$D/qrels" "$D/results"
printf '1 map 0 0\n' > "$T/zscores"
expect_output "$D/out.missing" -q -Z "$T/zscores" -m map "$D/qrels" "$D/results"
