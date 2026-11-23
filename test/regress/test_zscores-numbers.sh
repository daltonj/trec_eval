# Zscore means are finite numbers, and stddevs finite and non-negative.
# A valid first row checks that lines are counted.
. test/regress/lib.sh
D=test/regress/zscores-numbers
printf '1 0 d 1\n' > "$T/qrels"
printf '1 Q0 d 1 1 r\n' > "$T/results"

# refused ROW FIELD VALUE [ERROR]: the zscores row ROW, as line 2, is
# refused for its FIELD VALUE, after the conversion message ERROR if given.
refused() {
    printf '2 map 0 1\n%s\n' "$1" > "$T/zscores"
    where="in line 2 of zscores file '$T/zscores'"
    {
        if [ -n "$4" ]; then echo "$4 ($2 $where)."; fi
        echo "trec_eval.get_zscores: Unusable $2 '$3' $where"
        echo "trec_eval: Quit in file '$T/zscores'"
    } > "$T/want"
    expect_result 2 /dev/null "$T/want" -Z "$T/zscores" -m map \
        "$T/qrels" "$T/results"
}
refused '1 map abc 1' mean abc "Error: value \`abc' could not be parsed"
refused '1 map nan 1' mean nan "Error: value \`nan' is not a number"
refused '1 map inf 1' mean inf
refused '1 map 0 abc' stddev abc "Error: value \`abc' could not be parsed"
refused '1 map 0 inf' stddev inf
refused '1 map 0 -1' stddev -1

# A negative mean, a subnormal stddev and a zero stddev are valid. With a
# zero stddev, the value at the mean gives 0.
printf '1 map -1 2\n' > "$T/zscores"
expect_output $D/out.one -q -Z "$T/zscores" -m map "$T/qrels" "$T/results"
printf '1 map 1 1e-310\n' > "$T/zscores"
expect_status 0 -Z "$T/zscores" -m map "$T/qrels" "$T/results"
printf '1 map 1 0\n' > "$T/zscores"
expect_output $D/out.zero -q -Z "$T/zscores" -m map "$T/qrels" "$T/results"
