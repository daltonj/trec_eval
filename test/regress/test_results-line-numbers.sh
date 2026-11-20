# Messages about a line of a results file give its line number in the
# file, comment and blank lines included. In each fixture, line 5 is the
# line at fault and follows a comment, a blank line and an indented
# comment.
. test/regress/lib.sh
D=test/regress/results-line-numbers

# check RUN STATUS MESSAGE
check() {
    run=$1 status=$2 msg=$3
    expect_status $status -m num_ret $D/qrels $run
    "$TREC_EVAL" -m num_ret $D/qrels $run 2>&1 >/dev/null |
        grep -q "$msg" ||
        fail "trec_eval $D/qrels $run: no message naming line 5"
}

check $D/run.malformed 2 "Malformed line 5\$"
check $D/run.score 2 "value \`abc' could not be parsed (line 5 of results file '$D/run.score').\$"
check $D/run.trail 0 "value \`1.5x' partially read as \`1.5' (line 5 of results file '$D/run.trail').\$"
exit 0
