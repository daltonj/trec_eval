# get_qrels.c documents relevance levels from -127 to 127. Levels at the
# bounds are accepted. A level outside them is refused with exit 2 by the
# qrels and qrels_jg readers, before it can size the per-grade table.
. test/regress/lib.sh
D=test/regress/grade-range

expect_output $D/out.edge -q -m map -m ndcg $D/qrels.edge $D/results
expect_output $D/out.edge.jg -R qrels_jg -m map_avgjg $D/qrels.edge $D/results
for q in 128 -128 max; do
    case $q in
    max) level=2147483647 ;;
    *) level=$q ;;
    esac
    message="Relevance level '$level' for document 'd1' of topic '2' in line 2 of qrels file '$D/qrels.$q' is outside -127 to 127"
    expect_message 2 "$message" -m map $D/qrels.$q $D/results
    expect_message 2 "$message" -R qrels_jg -m map_avgjg $D/qrels.$q $D/results
done
expect_status 2 -m map $D/qrels.long64 $D/results
expect_status 2 -R qrels_jg -m map_avgjg $D/qrels.long64 $D/results
