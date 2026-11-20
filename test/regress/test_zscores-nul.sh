# Refuse a NUL within a field or after a complete row, with its line number.
. test/regress/lib.sh
D=test/regress/zscores-nul
T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

printf '1 map\000 0 1\n' > "$T/inside"
printf '1 map 0 1\n\000\n' > "$T/after"
line=1
for z in inside after; do
    expect_status 2 -Z "$T/$z" -m map "$D/qrels" "$D/results"
    "$TREC_EVAL" -Z "$T/$z" -m map "$D/qrels" "$D/results" \
        > "$T/out" 2> "$T/err"
    [ ! -s "$T/out" ] || fail "NUL input printed scores"
    printf "trec_eval.get_zscores: NUL character in line %s of zscores file '%s'\ntrec_eval: Quit in file '%s'\n" \
        "$line" "$T/$z" "$T/$z" > "$T/want"
    diff "$T/want" "$T/err" || fail "$z: wrong NUL diagnostic"
    line=2
done
