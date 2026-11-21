# With no -m, the measures are the default set for the rel_info format:
# official for qrels, qrels_jg for -R qrels_jg, and prefs for -R prefs and
# -R qrels_prefs.
. test/regress/lib.sh
Q=test/regress/qrels
P=test/regress/prefs
R=test/regress/results

save_output "$T/out" -R qrels_jg -m qrels_jg $Q $R
expect_output "$T/out" -R qrels_jg $Q $R
save_output "$T/out" -R prefs -m prefs $P $R
expect_output "$T/out" -R prefs $P $R
save_output "$T/out" -R qrels_prefs -m prefs $Q $R
expect_output "$T/out" -R qrels_prefs $Q $R
