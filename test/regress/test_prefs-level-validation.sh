# Both preference readers check numeric grades as the qrels reader does. A
# suffix warns and the parsed prefix is used (the .1x inputs must give
# grade-1 values).
. test/regress/lib.sh
D=test/regress/prefs-level-validation
for format in qrels_prefs prefs; do
    kind=qrels
    [ "$format" = prefs ] && kind=prefs
    where="document 'd1' of topic '1' in line 1 of $kind file"
    {
        echo "Error: value \`abc' could not be parsed ($where '$D/$kind.abc')."
        echo "trec_eval: Unusable relevance level 'abc'"
        echo "trec_eval: Quit in file '$D/$kind.abc'"
    } > "$T/err"
    expect_result 2 /dev/null "$T/err" -R $format $D/$kind.abc test/regress/results
    echo "Warning, value \`1x' partially read as \`1' ($where '$D/$kind.1x')." > "$T/err"
    expect_result 0 $D/out.grade1 "$T/err" -R $format \
        -m prefs_num_prefs_poss -m prefs_simp $D/$kind.1x test/regress/results
done
