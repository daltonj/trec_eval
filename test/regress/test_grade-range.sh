# get_qrels.c documents relevance levels from -127 to 127. Levels at the
# bounds are accepted. A level outside them is refused with exit 2 by the
# qrels and qrels_jg readers, before it can size the per-grade table.
. test/regress/lib.sh
d=test/regress/grade-range

expect_output $d/out.edge -q -m map -m ndcg $d/qrels.edge $d/results
expect_output $d/out.edge.jg -R qrels_jg -m map_avgjg $d/qrels.edge $d/results
for q in 128 -128 max; do
    m="in line 2 of qrels file '$d/qrels.$q' is outside -127 to 127"
    expect_message 2 "$m" -m map $d/qrels.$q $d/results
    expect_message 2 "$m" -R qrels_jg -m map_avgjg $d/qrels.$q $d/results
done
expect_status 2 -m map $d/qrels.long64 $d/results
expect_status 2 -R qrels_jg -m map_avgjg $d/qrels.long64 $d/results
