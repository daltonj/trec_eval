# A zscores file (-Z) that cannot be read, or has a malformed line, stops
# trec_eval with exit status 2 and "Quit in file", as for the judgments
# and results files.
. test/regress/lib.sh
d=test/regress/zscores-exit-status
Q=$T/qrels
R=$T/results
printf '1 0 d1 1\n1 0 d2 0\n2 0 d1 1\n' > "$Q"
printf '1 Q0 d1 1 2 r\n1 Q0 d2 2 1 r\n2 Q0 d1 1 1 r\n' > "$R"

for z in malformed missing; do
    expect_result 2 /dev/null $d/err.$z -q -Z $d/zscores.$z -m map "$Q" "$R"
done

expect_output $d/out.valid -q -Z test/regress/zscores -m map "$Q" "$R"
