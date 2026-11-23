# With no -m, each relevance format selects its own default measure set.
. test/regress/lib.sh
while read -r format input measures; do
    save_output "$T/out" -R $format -m $measures test/regress/$input test/regress/results
    expect_output "$T/out" -R $format test/regress/$input test/regress/results
done <<'EOF'
qrels_jg qrels qrels_jg
qrels_prefs qrels prefs
prefs prefs prefs
EOF
