# With no positive ideal gain, ndcg_rel and Rndcg must be 0. Document a is
# judged and x is not. ndcg_rel gives a the gain -0.5 or -3, and x the gain
# 1, which has no ideal occurrence; Rndcg gives a the gain 0. Topic 1
# retrieves x before a, topic 3 the reverse. Gain -0.5 leaves positive
# retrieved DCG after the ideal ranking ends; gain -3 makes it negative
# before the final correction.
. test/regress/lib.sh
D=test/regress/zero-ideal-gain
printf '1 0 a 1\n3 0 a 1\n' > "$T/qrels"
printf '1 Q0 x 1 2 r\n1 Q0 a 2 1 r\n3 Q0 a 1 2 r\n3 Q0 x 2 1 r\n' > "$T/results"
expect_output $D/out.zero -q -m ndcg_rel.-1=1,1=-0.5 -m Rndcg.1=0 \
    "$T/qrels" "$T/results"
expect_output $D/out.negative -q -m ndcg_rel.-1=1,1=-3 "$T/qrels" "$T/results"
