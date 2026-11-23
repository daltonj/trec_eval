# Checked conversions name their input. Numeric suffixes warn and retain
# the parsed prefix; unusable and non-finite parameters fail at init.
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
for format in qrels qrels_jg; do
    expect_message 0 "partially read as \`1' (document 'd1' of topic '1' in line 1 of qrels file 'test/regress/conversion-messages/qrels.partial')." \
        -R $format test/regress/conversion-messages/qrels.partial $R
done
printf '1 0 d1 abc\n' > "$T/abc.qrels"
where="document 'd1' of topic '1' in line 1 of qrels file '$T/abc.qrels'"
for format in qrels qrels_jg; do
    {
        echo "Error: value \`abc' could not be parsed ($where)."
        echo "trec_eval: Unusable relevance level 'abc'"
        echo "trec_eval: Quit in file '$T/abc.qrels'"
    } > "$T/want"
    expect_result 2 /dev/null "$T/want" -R $format "$T/abc.qrels" $R
done
expect_message 0 "Warning, value \`.25x' partially read as \`.25' (cutoff of measure 'iprec_at_recall')." \
    -m iprec_at_recall..25x $Q $R
te_same output test/regress/conversion-messages/out.partial
expect_message 0 "Warning, value \`0.5x' partially read as \`0.5' (parameter of measure 'set_F')." \
    -m set_F.0.5x $Q $R
expect_message 0 "Warning, value \`2x' partially read as \`2' (parameter '1' of measure 'ndcg')." \
    -m ndcg.1=2x $Q $R
