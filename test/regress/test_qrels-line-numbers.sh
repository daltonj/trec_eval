# Malformed-line messages count skipped comments.
. test/regress/lib.sh
D=test/regress/qrels-line-numbers
for format in qrels qrels_jg qrels_prefs prefs; do
    input=qrels
    [ "$format" = prefs ] && input=prefs
    expect_message 2 "Malformed line 3" -R $format $D/$input test/regress/results
done
