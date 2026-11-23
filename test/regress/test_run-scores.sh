# NaN is refused as a run score, and explicit infinities are valid. A score
# outside the range of a double is kept as strtod reads it, +-inf or +-0,
# with a warning that names the line. Scores that become equal tie, and the
# tie is broken by docno: b, a, c for 1e401, 1e400 and -1e400, and c, b, a
# for 2e-400, 1e-400 and -1e-400. A subnormal score is kept without a
# warning.
. test/regress/lib.sh
D=test/regress/run-scores
m="-q -m relstring test/regress/qrels.abc"

expect_message 2 "value \`nan' is not a number (line 2 of results file '$D/results.nan')." \
    $m $D/results.nan
expect_output $D/out.inf $m $D/results.inf
expect_result 0 $D/out.over $D/err.over $m $D/results.over
expect_result 0 $D/out.under $D/err.under $m $D/results.under
expect_result 0 $D/out.subnormal /dev/null $m $D/results.subnormal
