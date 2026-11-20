# A run score outside the range of a double is kept as strtod reads it,
# +-inf or +-0, with a warning that names the line. Scores that become
# equal tie, and the tie is broken by docno: b, a, c for 1e401, 1e400 and
# -1e400, and c, b, a for 2e-400, 1e-400 and -1e-400. A subnormal score
# is kept without a warning. The line number counts comment lines, as in
# the other results messages.
. test/regress/lib.sh
D=test/regress/run-score-range

m="-q -m relstring $D/qrels"
w="out of range, read as"

expect_result 0 $D/out.over $D/err.over $m $D/run.over
expect_result 0 $D/out.under $D/err.under $m $D/run.under
expect_message 0 "value \`1e400' $w \`inf' (line 2 of" $m $D/run.comment
expect_result 0 $D/out.subnormal /dev/null $m $D/run.subnormal
