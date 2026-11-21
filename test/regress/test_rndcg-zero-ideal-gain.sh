# Rndcg is 0, not NaN, for a topic whose relevant documents all have
# gain 0. Topic 1 is judged 0 only; topic 2 has one document of grade 1,
# ranked first.
. test/regress/lib.sh
D=test/regress/rndcg-zero-ideal-gain

# At -l 0 both documents of topic 1 are relevant, with gain 0.
expect_output $D/out.half -q -l 0 -m Rndcg $D/qrels $D/results
# The default level gives the same values: topic 1 has no relevant docs.
expect_output $D/out.half -q -m Rndcg $D/qrels $D/results
# A gain map that gives grade 1 gain 0 leaves topic 2 without ideal gain.
expect_output $D/out.zero -q -m Rndcg.1=0 $D/qrels $D/results
