# A large cutoff keeps its full row name and both decimals.
. test/regress/lib.sh
expect_output test/regress/cutoff-row-names/out.Rprec_mult \
    -m Rprec_mult.100000000 test/qrels.test test/results.test
