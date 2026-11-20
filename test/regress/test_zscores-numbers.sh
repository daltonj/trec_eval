# Zscores need finite means and finite, non-negative standard deviations.
. test/regress/lib.sh
D=test/regress/zscores-numbers
T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

for field in mean stddev; do
    values='abc nan inf -inf 1e999'
    [ "$field" = mean ] || values="$values -1"
    for value in $values; do
        if [ "$field" = mean ]; then
            printf '1 map 0 1\n1 map %s 1\n' "$value" > "$T/zscores"
        else
            printf '1 map 0 1\n1 map 0 %s\n' "$value" > "$T/zscores"
        fi
        expect_status 2 -Z "$T/zscores" -m map "$D/qrels" "$D/results"
        "$TREC_EVAL" -Z "$T/zscores" -m map "$D/qrels" "$D/results" \
            > "$T/out" 2> "$T/err"
        [ ! -s "$T/out" ] || fail "invalid $field printed scores"
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
        diff "$T/want" "$T/err" || fail "$field=$value: wrong diagnostic"
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
