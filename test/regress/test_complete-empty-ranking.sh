# Topics 1 and 3 are absent: exercise an empty ranking before allocation
# and after a nonempty ranking. With N=0 and one relevant doc, missing
# utility is 2*1 + 3*(0-1) = -1; num_ret/residual are 0, relstring is empty.
. test/regress/lib.sh
D=test/regress/complete-empty-ranking
expect_output $D/out -c -q -m num_ret -m utility.1,-1,2,3 \
    -m rbp_resid.p=0.5 -m relstring.3 $D/qrels $D/results
