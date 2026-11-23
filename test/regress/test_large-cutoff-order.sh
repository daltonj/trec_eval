# Integer cutoffs differing by more than INT_MAX still sort correctly.
# The run has one relevant doc, so precision at a huge cutoff rounds to 0.
. test/regress/lib.sh
need_long64
expect_output test/regress/large-cutoff-order/out.P \
    -m P.4294967297,1 test/regress/qrels.one test/regress/results.one
