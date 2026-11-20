# A results file with no data lines, or empty standard input, is refused
# with exit 2 and a message naming the file.
. test/regress/lib.sh
D=test/regress/empty-run-crash
Q=test/regress/qrels

for run in $D/run.blank $D/run.comment; do
    expect_message 2 "No data lines in results file '$run'" $Q $run
done
expect_message 2 "No data lines in results file '-'" $Q - < /dev/null
