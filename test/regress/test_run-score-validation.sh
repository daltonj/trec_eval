# A run score that is not a number, or is NaN, is refused with a message
# naming the line. Characters after a number give a warning and the number
# is used. inf and -inf are accepted.
. test/regress/lib.sh
D=test/regress/run-score-validation

m="-q -m relstring test/regress/qrels.abc"

expect_message 2 "value \`nan' is not a number (line 2 of results file '$D/results.nan')." \
    $m $D/results.nan
expect_message 2 "value \`abc' could not be parsed (line 2 of results file '$D/results.abc')." \
    $m $D/results.abc

echo "Warning, value \`1.5x' partially read as \`1.5' (line 2 of results file '$D/results.trail')." \
    > "$T/err"
expect_result 0 $D/out.trail "$T/err" $m $D/results.trail

expect_output $D/out.inf $m $D/results.inf
