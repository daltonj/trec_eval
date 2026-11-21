# A warning or error from reading a number says where the value comes
# from: the measure and its parameter, or the line of the file.
. test/regress/lib.sh
D=test/regress/conversion-messages
Q=test/regress/qrels
R=test/regress/results

w="Warning, value \`1.5' partially read as \`1'"
expect_message 0 "$w (cutoff of measure 'P')." -m P.1.5 $Q $R
expect_message 0 "$w (document 'd1' of topic '1' in line 1 of qrels file '$D/qrels.partial')." \
    -m num_rel $D/qrels.partial $R
expect_message 0 "$w (document 'd1' of topic '1' in line 1 of qrels file '$D/qrels.partial')." \
    -R qrels_jg -m map_avgjg $D/qrels.partial $R

# Floating-point parameters: cutoffs, plain parameters and name=value
# pairs.
expect_message 0 "Warning, value \`.25x' partially read as \`.25' (cutoff of measure 'iprec_at_recall')." \
    -m iprec_at_recall..25x $Q $R
expect_message 0 "Warning, value \`0.5x' partially read as \`0.5' (parameter of measure 'set_F')." \
    -m set_F.0.5x $Q $R
expect_message 0 "Warning, value \`2x' partially read as \`2' (parameter '1' of measure 'ndcg')." \
    -m ndcg.1=2x $Q $R
expect_message 0 "Warning, value \`0.5x' partially read as \`0.5' (parameter 'p' of measure 'rbp')." \
    -m rbp.p=0.5x $Q $R

# Errors carry the same context.
expect_message 2 "Error: value \`x' could not be parsed (cutoff of measure 'P')." \
    -m P.x $Q $R
expect_message 2 "Error: value \`x' could not be parsed (parameter of measure 'set_F')." \
    -m set_F.x $Q $R
expect_message 2 "trec_eval: cutoff inf of measure 'iprec_at_recall' is not finite" \
    -m iprec_at_recall.inf $Q $R
