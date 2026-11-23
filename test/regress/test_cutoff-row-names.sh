# A cutoff's row name keeps all its integer digits and two decimals, and
# distinct cutoffs must give distinct names.
. test/regress/lib.sh
Q=test/qrels.test
R=test/results.test
expect_message 2 "cutoffs 0.999 and 1.001 both give the name 'Rprec_mult_1.00'" \
    -m Rprec_mult.0.999,1.001 $Q $R
expect_output test/regress/cutoff-row-names/out \
    -m Rprec_mult.0.995,1.004,100000000 -m iprec_at_recall.0.1,0.106 $Q $R
