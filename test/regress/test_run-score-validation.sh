# A run score that is not a number, or is NaN, is refused with a message
# naming the line. Characters after a number give a warning and the number
# is used. inf and -inf are accepted.
. test/regress/lib.sh
D=test/regress/run-score-validation

m="-q -m relstring $D/qrels"

expect_message 2 "value \`nan' is not a number (line 2 of results file '$D/run.nan')." \
    $m $D/run.nan
expect_message 2 "value \`abc' could not be parsed (line 2 of results file '$D/run.abc')." \
    $m $D/run.abc

expect_output $D/out.trail $m $D/run.trail
expect_message 0 "value \`1.5x' partially read as \`1.5' (line 2 of results file" \
    $m $D/run.trail

expect_output $D/out.inf $m $D/run.inf
