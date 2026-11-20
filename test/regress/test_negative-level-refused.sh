# A negative -l is refused like an unparsable one: exit 1, no output.
# At -l -1, documents outside the qrels counted as relevant, but num_rel
# did not count them: here map was 3 and yaap was nan. -l 0 is still
# accepted.
. test/regress/lib.sh
d=test/regress/negative-level-refused
m="-q -m num_rel -m map -m yaap $d/qrels $d/results"

for l in "-l -1" -l-2 --level_for_rel=-1; do
    expect_output_status 1 $d/out.empty $l $m
done
expect_message 1 "trec_eval: Unusable relevance level '-1'" -l -1 $m

expect_output $d/out.l0 -l 0 $m
