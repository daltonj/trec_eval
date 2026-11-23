# te_calc_rbp rescales gains to [0,1] using a range that starts at 0.
# With p = 0.9, rbp = 0.1 * sum over ranks of 0.9^(rank-1) * gain.
. test/regress/lib.sh
D=test/regress/rbp-gain-range

# The gain range starts at 0, rather than at the first gain.
# Gains 1 (grade 0) and 3 (grade 1) rescale to 1/3 and 1, and the grade 1
# document is at rank 1: 0.1 * (1 + 0.9 * 1/3) = 0.13.
expect_output $D/out.two.gains13 -m rbp.0=1,1=3 $D/qrels.two $D/results.two

# Topic maxima differ (3 then 2), but each top document rescales to 1:
# 0.1 * 1.
# Keep the no-q call: it exposes stale stack values on some compilers.
expect_output $D/out.one -m rbp $D/qrels.topics $D/results.topics

# p can appear among gain pairs; both gains rescale to 1, so p=0.5
# gives (1-0.5)*(1+0.5)=0.75.
expect_result 0 $D/out.two.p /dev/null -m rbp.0=2,p=0.5,1=2 \
    $D/qrels.two $D/results.two
