# These messages from judgments readers and ranking formers start with
# trec_eval.<name>, where <name>.c is the file that prints them. A results
# file that retrieves d1 twice exits 4, and the duplicate-docs message,
# which each of the qrels, qrels_jg and qrels_prefs formats prints from its
# own code, ends its line before the "Can't calculate" message.
. test/regress/lib.sh
D=test/regress/message-prefixes
Q=test/regress/qrels
R=test/regress/results
printf '1 0 d1 1\n1 0 d1 0\n' > "$T/qrels.dup"
printf '1 Q0 d1 1 2 r\n1 Q0 d1 2 1 r\n' > "$T/results.dup"

expect_message 2 "trec_eval.get_qrels_prefs: Cannot read qrels file '$T/missing'" \
    -R qrels_prefs -m prefs_num_prefs_poss "$T/missing" $R
expect_message 2 "trec_eval.get_qrels_jg: duplicate docs d1" \
    -R qrels_jg -m map_avgjg "$T/qrels.dup" $R
expect_message 4 "trec_eval.form_res_rels: rel_info format not qrels" \
    -R qrels_jg -m map $Q $R
expect_message 4 "trec_eval.form_prefs_counts: prefs_info format not" \
    -m prefs_num_prefs_poss $Q $R
expect_result 4 /dev/null $D/err -m map test/regress/qrels.one "$T/results.dup"
expect_result 4 /dev/null $D/err.jg -R qrels_jg -m map_avgjg \
    test/regress/qrels.one "$T/results.dup"
expect_result 4 /dev/null $D/err.prefs -R qrels_prefs -m prefs_num_prefs_poss \
    test/regress/qrels.one "$T/results.dup"
