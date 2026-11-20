# Floating-point measure parameters that are not numbers are refused with
# exit status 2, as integer cutoffs are (-m P.x).
. test/regress/lib.sh
D=test/regress/float-parameters
Q=test/regress/qrels
R=test/regress/results

for m in iprec_at_recall.x Rprec_mult.x set_F.x 11pt_avg.x \
         iprec_at_recall..1,,.2 ndcg.1=x rbp.p=x; do
    expect_status 2 -m $m $Q $R
done

# Values such as .25, 1. and 0.5 are numbers, in lists and in gain pairs.
expect_output $D/out.valid -m iprec_at_recall..25,1.,0.5 \
    -m Rprec_mult.1.,.5 -m set_F.0.5 -m 11pt_avg.0.5,1. \
    -m ndcg.1=3,2=9,4=4.5 $Q $R

# Trailing characters after a number give a warning; the number is used.
expect_output $D/out.partial -m iprec_at_recall..25x $Q $R
