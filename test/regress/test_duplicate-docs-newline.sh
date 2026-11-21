# A results file that retrieves d1 twice for topic 1 cannot be evaluated:
# trec_eval exits 4, and the duplicate-docs message ends its own line
# before the "Can't calculate" message. The qrels, qrels_jg and
# qrels_prefs formats each detect the duplicate in their own code.
. test/regress/lib.sh
D=test/regress/duplicate-docs-newline
Q="$T/qrels"
R="$T/results"
printf '1 0 d1 1\n1 0 d2 0\n' > "$Q"
printf '1 Q0 d1 1 2 r\n1 Q0 d1 2 1 r\n' > "$R"

expect_result 4 /dev/null $D/err -m map "$Q" "$R"
expect_result 4 /dev/null $D/err.jg -R qrels_jg -m map_avgjg "$Q" "$R"
expect_result 4 /dev/null $D/err.prefs -R qrels_prefs -m prefs_num_prefs_poss \
    "$Q" "$R"
