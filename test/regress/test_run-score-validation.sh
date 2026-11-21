# NaN is refused; explicit infinities are valid run scores. Malformed and
# partially read scores are checked with physical line numbers separately.
. test/regress/lib.sh
D=test/regress/run-score-validation
m="-q -m relstring test/regress/qrels.abc"
expect_message 2 "value \`nan' is not a number (line 2 of results file '$D/results.nan')." \
    $m $D/results.nan
expect_output $D/out.inf $m $D/results.inf
