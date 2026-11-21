# A missing or malformed zscores file exits 2 and names the file.
. test/regress/lib.sh
D=test/regress/zscores-exit-status
for z in malformed missing; do
    expect_result 2 /dev/null $D/err.$z -Z $D/zscores.$z -m map \
        test/regress/qrels test/regress/results
done
