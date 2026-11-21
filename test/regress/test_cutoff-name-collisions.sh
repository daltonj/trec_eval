# Two cutoffs of iprec_at_recall, Rprec_mult or Rprec_mult_avgjg that give
# the same printed name are refused with exit status 2. Names show a
# cutoff to two decimals, so 0.999 and 1.001 both give Rprec_mult_1.00.
# Cutoffs whose names differ are accepted.
. test/regress/lib.sh
D=test/regress/cutoff-name-collisions
Q=test/qrels.test
R=test/results.test

expect_message 2 "cutoffs 0.999 and 1.001 both give the name 'Rprec_mult_1.00'" \
    -m Rprec_mult.0.999,1.001 $Q $R
expect_message 2 "cutoffs 0.101 and 0.104 both give the name 'iprec_at_recall_0.10'" \
    -m iprec_at_recall.0.101,0.104 $Q $R
expect_message 2 "cutoffs 0.999 and 1.001 both give the name 'Rprec_mult_avgjg_1.00'" \
    -R qrels_jg -m Rprec_mult_avgjg.0.999,1.001 test/qrels.rel_level $R

expect_output $D/out.distinct -m Rprec_mult.0.995,1.004 \
    -m iprec_at_recall.0.1,0.106 $Q $R
