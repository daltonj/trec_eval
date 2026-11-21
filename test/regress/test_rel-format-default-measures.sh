# With no -m, each relevance format selects its own default measure set.
. test/regress/lib.sh
for format in qrels_jg prefs qrels_prefs; do
    input=qrels
    measures=prefs
    [ "$format" = prefs ] && input=prefs
    [ "$format" = qrels_jg ] && measures=qrels_jg
    save_output "$T/out" -R $format -m $measures test/regress/$input test/regress/results
    expect_output "$T/out" -R $format test/regress/$input test/regress/results
done
