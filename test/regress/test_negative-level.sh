# A negative -l is refused like an unparsable one: exit 1, no output.
# -l 0 is accepted.
. test/regress/lib.sh
d=test/regress/negative-level
m="-q -m num_rel -m map -m yaap $d/qrels $d/results"

for l in "-l -1" -l-2 --level_for_rel=-1; do
    expect_output_status 1 /dev/null $l $m
done
expect_message 1 "trec_eval: Unusable relevance level '-1'" -l -1 $m

expect_output $d/out.l0 -l 0 $m
