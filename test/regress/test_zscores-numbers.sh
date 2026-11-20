# Zscores need finite means and finite, non-negative standard deviations.
. test/regress/lib.sh
D=test/regress/zscores-numbers
T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

for field in mean stddev; do
    values='abc 1x nan inf -inf 1e999'
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
        printf "trec_eval.get_zscores: Unusable %s '%s' in line 2\ntrec_eval: Quit in file '%s'\n" \
            "$field" "$value" "$T/zscores" > "$T/want"
        diff "$T/want" "$T/err" || fail "$field=$value: wrong diagnostic"
    done
done

for row in '1 map -1 2' '1 map 1 0' '1 map 0 0' '1 map 1 1e-310'; do
    printf '%s\n' "$row" > "$T/zscores"
    expect_status 0 -Z "$T/zscores" -m map "$D/qrels" "$D/results"
done
printf '1 map -1 2\n' > "$T/zscores"
expect_output "$D/out.one" -q -Z "$T/zscores" -m map "$D/qrels" "$D/results"
printf '1 map 1 0\n' > "$T/zscores"
expect_output "$D/out.zero" -q -Z "$T/zscores" -m map "$D/qrels" "$D/results"
printf '1 map 0 0\n' > "$T/zscores"
expect_output "$D/out.missing" -q -Z "$T/zscores" -m map "$D/qrels" "$D/results"
