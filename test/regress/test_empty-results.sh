# No-data results, including empty stdin, fail before topic evaluation.
. test/regress/lib.sh
printf '# comment\n \t \n' > "$T/results"
expect_message 2 "No data lines in results file '$T/results'" test/regress/qrels "$T/results"
expect_message 2 "No data lines in results file '-'" test/regress/qrels - < /dev/null
