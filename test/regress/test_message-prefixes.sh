# These six messages from judgments readers and ranking formers start
# with trec_eval.<name>, where <name>.c is the file that prints them.
. test/regress/lib.sh
D=test/regress/message-prefixes

expect_message 2 "trec_eval.get_qrels_prefs: Cannot read qrels file '$D/missing'" \
    -R qrels_prefs -m prefs_num_prefs_poss $D/missing $D/results
expect_message 2 "trec_eval.get_qrels_jg: duplicate docs d1" \
    -R qrels_jg -m map_avgjg $D/qrels.dup $D/results
expect_message 4 "trec_eval.form_res_rels: rel_info format not qrels" \
    -R qrels_jg -m map $D/qrels $D/results
expect_message 4 "trec_eval.form_res_rels: duplicate docs d1" \
    -m map $D/qrels $D/results.dup
expect_message 4 "trec_eval.form_res_rels_jg: duplicate docs d1" \
    -R qrels_jg -m map_avgjg $D/qrels $D/results.dup
expect_message 4 "trec_eval.form_prefs_counts: prefs_info format not" \
    -m prefs_num_prefs_poss $D/qrels $D/results
