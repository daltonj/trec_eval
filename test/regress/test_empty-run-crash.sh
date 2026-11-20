# A results file with no data lines is refused with exit 2 and a message
# naming the file; it used to crash.
. test/regress/lib.sh
D=test/regress/empty-run-crash

for run in $D/run.blank $D/run.comment; do
    expect_message 2 "No data lines in results file '$run'" $D/qrels $run
done
expect_message 2 "No data lines in results file '-'" $D/qrels - < /dev/null
