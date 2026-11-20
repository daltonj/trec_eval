# With no -m, the measures are the default set for the rel_info format:
# official for qrels, qrels_jg for -R qrels_jg, and prefs for -R prefs and
# -R qrels_prefs. The expected output was made with the -m option given.
. test/regress/lib.sh
d=test/regress/rel-format-default-measures

expect_output $d/out.qrels_jg -R qrels_jg $d/qrels_jg $d/results
expect_output $d/out.prefs -R prefs $d/prefs $d/results
expect_output $d/out.qrels_prefs -R qrels_prefs $d/qrels $d/results
