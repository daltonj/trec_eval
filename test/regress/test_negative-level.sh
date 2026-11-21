# A negative -l is refused like an unparsable one: exit 1, no output.
# -l 0 is accepted.
. test/regress/lib.sh
D=test/regress/negative-level
printf '1 0 a 1\n' > "$T/qrels"
printf '1 Q0 x 1 3 r\n1 Q0 y 2 2 r\n1 Q0 z 3 1 r\n' > "$T/results"
set -- -q -m num_rel -m map -m yaap "$T/qrels" "$T/results"

echo "trec_eval: Unusable relevance level '-1'" > "$T/err"
for l in "-l -1" -l-1 --level_for_rel=-1; do
    expect_result 1 /dev/null "$T/err" $l "$@"
done

expect_output $D/out.l0 -l 0 "$@"
