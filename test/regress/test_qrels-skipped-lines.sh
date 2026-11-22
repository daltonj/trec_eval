# Skip comments and white space, warning only for blank lines. A comment
# has data fields so that reading it as a topic changes the -c output.
. test/regress/lib.sh
R=test/regress/results
for format in qrels qrels_jg qrels_prefs prefs; do
    kind=qrels
    [ "$format" = prefs ] && kind=prefs
    {
        printf '  # x y z 1\n'
        head -n 1 test/regress/$kind
        printf '\n \t \n'
        tail -n +2 test/regress/$kind
        echo '# end'
    } > "$T/input"
    save_output "$T/plain" -c -q -R $format test/regress/$kind $R
    for line in 3 4; do
        printf "Warning, blank line skipped (line %s of %s file '%s').\n" \
            $line $kind "$T/input"
    done > "$T/warning"
    expect_result 0 "$T/plain" "$T/warning" -c -q -R $format "$T/input" $R
    printf '# comment\n\n' > "$T/only"
    expect_message 2 "No data lines in $kind file '$T/only'" \
        -R $format "$T/only" $R
done
