# A zscores file (-Z) that cannot be read, or has a malformed line, stops
# trec_eval with exit status 2 and "Quit in file", as for the judgments
# and results files.
. test/regress/lib.sh
d=test/regress/zscores-exit-status

for z in malformed missing; do
    expect_result 2 /dev/null $d/err.$z -q -Z $d/zscores.$z -m map \
        $d/qrels $d/results
done

expect_output $d/out.valid -q -Z $d/zscores -m map $d/qrels $d/results
