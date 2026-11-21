# A results file with no data lines, or empty standard input, is refused
# with exit 2 and a message naming the file.
. test/regress/lib.sh
D=test/regress/empty-results
Q=test/regress/qrels

for results in $D/results.blank $D/results.comment; do
    expect_message 2 "No data lines in results file '$results'" $Q $results
done
expect_message 2 "No data lines in results file '-'" $Q - < /dev/null
