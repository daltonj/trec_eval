# A malformed line in a qrels, qrels_jg or prefs file is reported by its
# line number in the file, comment lines included. In each fixture, line 4
# has no relevance field and follows two comment lines.
. test/regress/lib.sh
D=test/regress/qrels-line-numbers
R=test/regress/results

expect_message 2 "Malformed line 4" -m num_rel $D/qrels $R
expect_message 2 "Malformed line 4" -R qrels_jg -m map_avgjg $D/qrels $R
expect_message 2 "Malformed line 4" \
    -R qrels_prefs -m prefs_num_prefs_poss $D/qrels $R
expect_message 2 "Malformed line 4" \
    -R prefs -m prefs_num_prefs_poss $D/prefs $R
