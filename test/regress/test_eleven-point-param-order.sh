# Relevant documents at ranks 1 and 3 give interpolated precision 1 at
# recall 0.5 and 2/3 at recall 1: the average is 5/6, in either order.
. test/regress/lib.sh
D=test/regress/eleven-point-param-order
expect_output $D/out.down -m 11pt_avg.1.0,0.5 $D/qrels $D/results
