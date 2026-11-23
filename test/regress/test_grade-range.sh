# Both integer-grade readers accept -127..127 and refuse the adjacent values.
. test/regress/lib.sh
D=test/regress/grade-range
R=test/regress/results
printf '1 0 d1 127\n1 0 d2 -127\n' > "$T/edge"
expect_output $D/out.edge -q -m map -m ndcg "$T/edge" $R
expect_output $D/out.edge.jg -R qrels_jg -m map_avgjg "$T/edge" $R
for level in -128 128; do
    printf '1 0 d1 %s\n' "$level" > "$T/qrels"
    for format in qrels qrels_jg; do
        expect_message 2 "Relevance level '$level' for document 'd1' of topic '1' in line 1 of qrels file '$T/qrels' is outside -127 to 127" \
            -R $format "$T/qrels" $R
    done
done
# Validate before narrowing: this becomes -1 if stored in a 32-bit int.
# A 32-bit long must reject it during conversion instead.
printf '1 0 d1 9223372036854775807\n' > "$T/wide"
for format in qrels qrels_jg; do
    expect_status 2 -R $format "$T/wide" $R
done
