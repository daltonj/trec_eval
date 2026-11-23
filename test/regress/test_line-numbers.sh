# Messages about a line of a judgments or results file give its line
# number in the file, comment and blank lines included.
. test/regress/lib.sh
R=test/regress/results
printf '# comment\n1 0 d1 1\n1 0 d2\n' > "$T/qrels"
printf '# comment\n1 A 0 d1 1\n1 A 0 d2\n' > "$T/prefs"
while read -r format kind; do
    expect_message 2 "Malformed line 3" -R $format "$T/$kind" $R
done <<'EOF'
qrels qrels
qrels_jg qrels
qrels_prefs qrels
prefs prefs
EOF

# results LINE: write a results file whose line 5 is LINE, after a comment,
# a blank line, an indented comment and a valid line.
results() {
    printf '# one\n\n  # two\n1 Q0 a 1 1 r\n%s\n' "$1" > "$T/results"
}
Q=test/regress/qrels.abc
where="line 5 of results file '$T/results'"
results '1 Q0 b 2'
expect_message 2 "Malformed line 5" -m num_ret $Q "$T/results"
results '1 Q0 b 2 abc r'
expect_message 2 "value \`abc' could not be parsed ($where)." \
    -m num_ret $Q "$T/results"
results '1 Q0 b 2 1.5x r'
echo "Warning, value \`1.5x' partially read as \`1.5' ($where)." > "$T/want"
expect_result 0 test/regress/line-numbers/out.trail "$T/want" \
    -q -m relstring $Q "$T/results"
