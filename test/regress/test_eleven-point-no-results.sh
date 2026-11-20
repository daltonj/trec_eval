# 11pt_avg of a topic with nothing retrieved is 0, not nan. Under -J the
# unjudged documents x and y are removed, so topic 1 retrieves nothing;
# under -M 0 no topic retrieves anything.
. test/regress/lib.sh
d=test/regress/eleven-point-no-results
Q=$T/qrels
R=$T/results
printf '1 0 a 1\n1 0 b 0\n2 0 a 1\n' > "$Q"
printf '1 Q0 x 1 2 r\n1 Q0 y 2 1 r\n2 Q0 a 1 1 r\n' > "$R"
expect_output $d/out.J -J -q -m 11pt_avg "$Q" "$R"
expect_output $d/out.M0 -M 0 -q -m 11pt_avg "$Q" "$R"
# iprec_at_recall is 0 at every cutoff for such a topic. Under -J, topic 2
# is 1 at every cutoff.
expect_output $d/out.J.iprec -J -q -m iprec_at_recall "$Q" "$R"
expect_output $d/out.M0.iprec -M 0 -q -m iprec_at_recall "$Q" "$R"
