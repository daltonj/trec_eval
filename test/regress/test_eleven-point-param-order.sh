# Topic 1 has relevant documents at ranks 1 and 3, giving interpolated
# precision 1 at recall 0.5 and 2/3 at recall 1, an average of 5/6 in
# either order. Topic 2 has 1 at both, so the mean is 11/12 = 0.9167.
. test/regress/lib.sh
expect_output test/regress/eleven-point-param-order/out.down \
    -m 11pt_avg.1.0,0.5 test/regress/qrels test/regress/results
