# Topic 1 has only negative grades, so it has no relevant document and
# its ideal DCG is 0. Its score must not depend on topic 0, which comes
# before it and has one document at grade 1. The gain map gives the
# negative-grade documents gain 1, so topic 1 has a nonzero DCG.
. test/regress/lib.sh
D=test/regress/rel-levels-reset
Q="$T/qrels"
R="$T/results"
printf '0 0 d1 0\n0 0 d2 1\n1 0 d1 -1\n1 0 d2 -1\n' > "$Q"
printf '0 Q0 d2 1 2 r\n0 Q0 d1 2 1 r\n1 Q0 d1 1 3 r\n1 Q0 d2 2 2 r\n' > "$R"

expect_output $D/out.ndcg.q -q -m ndcg.-2=1 "$Q" "$R"
