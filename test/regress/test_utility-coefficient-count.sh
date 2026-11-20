# utility takes exactly four coefficients. Another number is refused when
# the measure is initialised, with exit 2 and no output, as for other
# unusable measure parameters. Before, it was refused only while scoring
# the first topic, with exit 4.
. test/regress/lib.sh
d=test/regress/utility-coefficient-count
Q=test/regress/qrels
R=test/regress/results

for p in 1,2 1,2,3,4,5; do
    expect_output_status 2 /dev/null -q -m utility.$p $Q $R
done
expect_message 2 "improper number of coefficients: 2 given, 4 needed" \
    -q -m utility.1,2 $Q $R
expect_message 2 "improper number of coefficients: 5 given, 4 needed" \
    -q -m utility.1,2,3,4,5 $Q $R

expect_output $d/out.valid -q -m utility.2,-1,0,0 $Q $R
