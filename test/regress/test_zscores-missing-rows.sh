# -Z prints -1000000 for a topic and measure that have no line in the
# zscores file. The zscores file has num_rel for topic 1 and num_ret for
# topic 2 only. Topic 1's num_ret must not be taken from topic 2's line.
. test/regress/lib.sh
printf '1 num_rel 1.0 1.0\n2 num_ret 0.0 1.0\n' > "$T/zscores"
expect_output test/regress/zscores-missing-rows/out -q -n -Z "$T/zscores" \
    -m num_ret -m num_rel test/regress/qrels test/regress/results
