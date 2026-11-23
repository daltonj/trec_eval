# Zscore numbers parse completely: means are finite, stddevs are finite
# and non-negative. A valid first row checks that lines are counted.
. test/regress/lib.sh
D=test/regress/zscores-numbers
printf '1 0 d 1\n' > "$T/qrels"
printf '1 Q0 d 1 1 r\n' > "$T/results"

# refused ROW FIELD: the zscores row ROW, as line 2, is refused for FIELD.
refused() {
    printf '2 map 0 1\n%s\n' "$1" > "$T/zscores"
    {
        printf "trec_eval.get_zscores: Unusable %s in line 2 " "$2"
        printf "of zscores file '%s'\n" "$T/zscores"
        printf "trec_eval: Quit in file '%s'\n" "$T/zscores"
    } > "$T/want"
    expect_result 2 /dev/null "$T/want" -Z "$T/zscores" -m map \
        "$T/qrels" "$T/results"
}
refused '1 map abc 1' "mean 'abc'"
refused '1 map 1x 1' "mean '1x'"
refused '1 map nan 1' "mean 'nan'"
refused '1 map inf 1' "mean 'inf'"
refused '1 map 0 abc' "stddev 'abc'"
refused '1 map 0 1x' "stddev '1x'"
refused '1 map 0 inf' "stddev 'inf'"
refused '1 map 0 -1' "stddev '-1'"

# A negative mean, a subnormal stddev and a zero stddev are valid. With a
# zero stddev, the value at the mean gives 0.
printf '1 map -1 2\n' > "$T/zscores"
expect_output $D/out.one -q -Z "$T/zscores" -m map "$T/qrels" "$T/results"
printf '1 map 1 1e-310\n' > "$T/zscores"
expect_status 0 -Z "$T/zscores" -m map "$T/qrels" "$T/results"
printf '1 map 1 0\n' > "$T/zscores"
expect_output $D/out.zero -q -Z "$T/zscores" -m map "$T/qrels" "$T/results"
