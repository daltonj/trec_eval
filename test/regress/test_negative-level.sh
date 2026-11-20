# A negative -l is refused like an unparsable one: exit 1, no output.
# -l 0 is accepted.
. test/regress/lib.sh
d=test/regress/negative-level
m="-q -m num_rel -m map -m yaap $d/qrels $d/results"

echo "trec_eval: Unusable relevance level '-1'" > "$T/err"
for l in "-l -1" -l-1 --level_for_rel=-1; do
    expect_result 1 /dev/null "$T/err" $l $m
done

expect_output $d/out.l0 -l 0 $m
