# A zscores file (-Z) that cannot be read, or has a malformed line, stops
# trec_eval with exit status 2 and "Quit in file", as for the judgments
# and results files. Before, main returned UNDEF and the status was 255.
. test/regress/lib.sh
d=test/regress/zscores-exit-status

for z in $d/zscores.malformed $d/no-such-file; do
    expect_output_status 2 $d/out.empty -q -Z $z -m map $d/qrels $d/results
    expect_message 2 "trec_eval: Quit in file '$z'" \
        -q -Z $z -m map $d/qrels $d/results
done

expect_output $d/out.valid -q -Z $d/zscores -m map $d/qrels $d/results
