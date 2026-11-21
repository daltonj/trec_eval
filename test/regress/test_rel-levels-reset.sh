# Topic 0 has grade 1; topic 1 has only a negative grade. The latter's
# ideal DCG must be 0 regardless of the cached grade count. Negative
# grades become RELVALUE_UNJUDGED (-2), given gain 1 so its DCG is nonzero.
. test/regress/lib.sh
printf '0 0 d 1\n1 0 d -1\n' > "$T/qrels"
printf '0 Q0 d 1 1 r\n1 Q0 d 1 1 r\n' > "$T/results"
expect_output test/regress/rel-levels-reset/out.ndcg.q \
    -q -m ndcg.-2=1 "$T/qrels" "$T/results"
