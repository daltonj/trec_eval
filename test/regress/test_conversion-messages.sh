# Checked conversions of measure parameters and grades name their input.
# Numeric suffixes warn and retain the parsed prefix; unusable and
# non-finite parameters fail at init.
. test/regress/lib.sh
Q=test/regress/qrels
R=test/regress/results
# utility.x,0,0,0 and ndcg.1=x,2=1 put the bad value before a comma, in
# each list reader.
for measure in iprec_at_recall.x iprec_at_recall..1,,.2 \
    11pt_avg.x ndcg.1=x utility.x,0,0,0 ndcg.1=x,2=1; do
    expect_message 2 "could not be parsed" -m $measure $Q $R
done
for measure in iprec_at_recall.inf set_F.inf ndcg.1=inf; do
    expect_message 2 "is not finite" -m $measure $Q $R
done
expect_message 2 "could not be parsed (cutoff of measure 'P')." -m P.x $Q $R
expect_message 0 "partially read as \`1' (cutoff of measure 'P')." -m P.1.5 $Q $R
echo "Warning, value \`.25x' partially read as \`.25' (cutoff of measure 'iprec_at_recall')." > "$T/want"
expect_result 0 test/regress/conversion-messages/out.partial "$T/want" \
    -m iprec_at_recall..25x $Q $R
expect_message 0 "Warning, value \`0.5x' partially read as \`0.5' (parameter of measure 'set_F')." \
    -m set_F.0.5x $Q $R
expect_message 0 "Warning, value \`2x' partially read as \`2' (parameter '1' of measure 'ndcg')." \
    -m ndcg.1=2x $Q $R

# Each judgments reader refuses the grade abc. It warns about the grade 1x
# and reads it as 1, so that the input scores as the one with grade 1.
while read -r format kind prefix; do
    where="document 'd1' of topic '1' in line 1 of $kind file"
    printf '%s d1 abc\n' "$prefix" > "$T/abc"
    {
        echo "Error: value \`abc' could not be parsed ($where '$T/abc')."
        echo "trec_eval: Unusable relevance level 'abc'"
        echo "trec_eval: Quit in file '$T/abc'"
    } > "$T/want"
    expect_result 2 /dev/null "$T/want" -R $format "$T/abc" $R
    printf '%s d1 1\n%s d2 0\n' "$prefix" "$prefix" > "$T/1"
    save_output "$T/out" -R $format "$T/1" $R
    printf '%s d1 1x\n%s d2 0\n' "$prefix" "$prefix" > "$T/1x"
    echo "Warning, value \`1x' partially read as \`1' ($where '$T/1x')." > "$T/want"
    expect_result 0 "$T/out" "$T/want" -R $format "$T/1x" $R
done <<'EOF'
qrels qrels 1 0
qrels_jg qrels 1 0
qrels_prefs qrels 1 0
prefs prefs 1 A 0
EOF
