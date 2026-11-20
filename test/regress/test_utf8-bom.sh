# A UTF-8 byte-order mark at the start of an input file is skipped, so a
# file gives the same values with or without one.
. test/regress/lib.sh
Q=test/regress/qrels
P=test/regress/prefs
R=test/regress/results

printf '1 map 0.5 0.25\n2 map 0.5 0.25\n' > "$T/zscores"
for f in $Q $P $R "$T/zscores"; do
    { printf '\357\273\277'; cat $f; } > "$T/${f##*/}.bom"
done
{ printf '\357\273\277# comment\n'; cat $Q; } > "$T/qrels.comment"

m="-c -q -m num_q -m num_ret -m num_rel -m map"
save_output "$T/plain" $m $Q $R
expect_output "$T/plain" $m "$T/qrels.bom" $R
expect_output "$T/plain" $m "$T/qrels.comment" $R
expect_output "$T/plain" $m $Q "$T/results.bom"
expect_output "$T/plain" $m $Q - < "$T/results.bom"
m="-q -R qrels_jg -m map_avgjg"
save_output "$T/plain" $m $Q $R
expect_output "$T/plain" $m "$T/qrels.bom" $R
m="-q -R qrels_prefs -m prefs_num_prefs_poss"
save_output "$T/plain" $m $Q $R
expect_output "$T/plain" $m "$T/qrels.bom" $R
m="-q -R prefs -m prefs_num_prefs_poss"
save_output "$T/plain" $m $P $R
expect_output "$T/plain" $m "$T/prefs.bom" $R
m="-q -m map"
save_output "$T/plain" $m -Z "$T/zscores" $Q $R
expect_output "$T/plain" $m -Z "$T/zscores.bom" $Q $R
