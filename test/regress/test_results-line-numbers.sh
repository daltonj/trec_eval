# Messages about a line of a results file give its line number in the
# file, comment and blank lines included. In each fixture, line 5 is the
# line at fault and follows a comment, a blank line and an indented
# comment.
. test/regress/lib.sh
D=test/regress/results-line-numbers

expect_message 2 "Malformed line 5" -m num_ret $D/qrels $D/results.malformed
expect_message 2 "value \`abc' could not be parsed (line 5 of results file '$D/results.score')." \
    -m num_ret $D/qrels $D/results.score
expect_message 0 "value \`1.5x' partially read as \`1.5' (line 5 of results file '$D/results.trail')." \
    -m num_ret $D/qrels $D/results.trail
