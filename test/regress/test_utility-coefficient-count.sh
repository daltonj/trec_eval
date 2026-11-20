# utility takes exactly four coefficients. Another number is refused when
# the measure is initialised, with exit 2 and no output, as for other
# unusable measure parameters.
. test/regress/lib.sh
d=test/regress/utility-coefficient-count
Q=test/regress/qrels
R=test/regress/results

expect_result 2 /dev/null $d/err.2 -q -m utility.1,2 $Q $R
expect_result 2 /dev/null $d/err.5 -q -m utility.1,2,3,4,5 $Q $R

expect_output $d/out.valid -q -m utility.2,-1,0,0 $Q $R
