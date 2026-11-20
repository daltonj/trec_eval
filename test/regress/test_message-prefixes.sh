# These messages from judgments readers and ranking formers start with
# trec_eval.<name>, where <name>.c is the file that prints them. The
# duplicate-docs messages of the ranking formers are checked whole in
# test_duplicate-docs-newline.sh.
. test/regress/lib.sh
Q=test/regress/qrels
R=test/regress/results
printf '1 0 d1 1\n1 0 d1 0\n' > "$T/qrels.dup"

expect_message 2 "trec_eval.get_qrels_prefs: Cannot read qrels file '$T/missing'" \
    -R qrels_prefs -m prefs_num_prefs_poss "$T/missing" $R
expect_message 2 "trec_eval.get_qrels_jg: duplicate docs d1" \
    -R qrels_jg -m map_avgjg "$T/qrels.dup" $R
expect_message 4 "trec_eval.form_res_rels: rel_info format not qrels" \
    -R qrels_jg -m map $Q $R
expect_message 4 "trec_eval.form_prefs_counts: prefs_info format not" \
    -m prefs_num_prefs_poss $Q $R
