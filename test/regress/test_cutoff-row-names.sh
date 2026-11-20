# A row name shows a floating-point cutoff in full, with two decimals:
# Rprec_mult.100000000 prints Rprec_mult_100000000.00, and 100000.01 and
# 100000.02 print names that differ.
. test/regress/lib.sh
D=test/regress/cutoff-row-names

expect_output $D/out.Rprec_mult \
    -m Rprec_mult.0.2,99999.99,100000.01,100000.02,100000000 \
    test/qrels.test test/results.test
expect_output $D/out.avgjg -R qrels_jg -m Rprec_mult_avgjg.0.2,100000000 \
    test/qrels.rel_level test/results.test
exit 0
