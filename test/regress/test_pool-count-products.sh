# Pool requests multiply counts in a long. Where long has 32 bits, as on
# Windows, 65536 * 65536 does not fit: trec_eval must refuse the request
# and exit 4 instead of filling a table that is too small for its counts.
. test/regress/lib.sh
need_long32

# One prefs judgment group of 65536 documents: num_judged * num_judged
awk 'BEGIN { for (d = 0; d < 65536; d++)
             printf "1 A 0 d%05d %d\n", d, (d != 65535) }' > "$T/prefs"
printf '1 Q0 d00000 1 1 r\n' > "$T/one.run"
expect_status 4 -R prefs -m prefs_num_prefs_poss "$T/prefs" "$T/one.run"
grep -q "Can't calculate measure 'prefs_num_prefs_poss'" "$T/te.error" ||
    te_fail "no refusal message"

# 65536 judgment groups and 65536 results: num_results * num_jgs
awk 'BEGIN { for (g = 0; g < 65536; g++)
             printf "1 A%05d d00000 1\n", g }' > "$T/jg"
awk 'BEGIN { for (d = 0; d < 65536; d++)
             printf "1 Q0 d%05d 1 1 r\n", d }' > "$T/many.run"
expect_status 4 -R qrels_jg -m map_avgjg "$T/jg" "$T/many.run"
grep -q "Can't calculate measure 'map_avgjg'" "$T/te.error" ||
    te_fail "no refusal message"
