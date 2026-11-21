# Both preference readers check numeric grades. A suffix warns and the
# parsed prefix is used (the .1x inputs must give grade-1 values).
. test/regress/lib.sh
D=test/regress/prefs-level-validation
for format in qrels_prefs prefs; do
    kind=qrels
    [ "$format" = prefs ] && kind=prefs
    expect_message 2 "could not be parsed (line 1 of $kind file '$D/$kind.abc')." \
        -R $format $D/$kind.abc test/regress/results
    echo "Warning, value \`1x' partially read as \`1' (line 1 of $kind file '$D/$kind.1x')." > "$T/err"
    expect_result 0 $D/out.grade1 "$T/err" -R $format \
        -m prefs_num_prefs_poss -m prefs_simp $D/$kind.1x test/regress/results
done
