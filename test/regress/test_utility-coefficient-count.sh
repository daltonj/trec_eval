# utility takes exactly four coefficients. Another number is refused when
# the measure is initialized, with exit 2 and no output, as for other
# unusable measure parameters.
. test/regress/lib.sh
D=test/regress/utility-coefficient-count
Q=test/regress/qrels
R=test/regress/results

expect_result 2 /dev/null $D/err.2 -q -m utility.1,2 $Q $R
expect_result 2 /dev/null $D/err.5 -q -m utility.1,2,3,4,5 $Q $R

expect_output $D/out.valid -q -m utility.2,-1,0,0 $Q $R
