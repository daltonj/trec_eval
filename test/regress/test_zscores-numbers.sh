# Zscore numbers parse completely: means are finite, stddevs are finite
# and non-negative. Prefix a valid row to check physical line numbers.
. test/regress/lib.sh
D=test/regress/zscores-numbers
printf '1 0 d 1\n' > "$T/qrels"
printf '1 Q0 d 1 1 r\n' > "$T/results"
for field in mean stddev; do
    values='abc 1x inf'
    [ "$field" = mean ] || values="$values -1"
    for value in $values; do
        printf '2 map 0 1\n' > "$T/zscores"
        case $field in
        mean) printf '1 map %s 1\n' "$value" ;;
        stddev) printf '1 map 0 %s\n' "$value" ;;
        esac >> "$T/zscores"
        loc="line 2 of zscores file '$T/zscores'"
        case $value in
        abc) message="Error: value \`abc' could not be parsed ($field in $loc)." ;;
        *) message="trec_eval.get_zscores: Unusable $field '$value' in $loc" ;;
        esac
        printf "%s\ntrec_eval: Quit in file '%s'\n" "$message" "$T/zscores" > "$T/want"
        expect_result 2 /dev/null "$T/want" -Z "$T/zscores" -m map "$T/qrels" "$T/results"
    done
done
# Negative means and subnormal stddevs are valid. Zero stddev yields 0
# at the mean, and -1000000 otherwise.
printf '1 map 1 1e-310\n' > "$T/zscores"
expect_status 0 -Z "$T/zscores" -m map "$T/qrels" "$T/results"
printf '1 map -1 2\n' > "$T/zscores"
expect_output $D/out.one -q -Z "$T/zscores" -m map "$T/qrels" "$T/results"
printf '1 map 1 0\n' > "$T/zscores"
expect_output $D/out.zero -q -Z "$T/zscores" -m map "$T/qrels" "$T/results"
printf '1 map 0 0\n' > "$T/zscores"
expect_output $D/out.missing -q -Z "$T/zscores" -m map "$T/qrels" "$T/results"
