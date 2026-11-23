# Every reader rejects NUL and reports its physical line, including comments.
. test/regress/lib.sh
Q=test/regress/qrels
R=test/regress/results
printf '# comment\n1 0 d1\000x 1\n' > "$T/qrels"
printf '# comment\n1 A 0 d1\000x 1\n' > "$T/prefs"
printf '# comment\n1 Q0 d1\000x 1 1 r\n' > "$T/results"
printf '1 map 0 1\n1 map\000 0 1\n' > "$T/zscores"

while read -r format kind; do
    expect_message 2 "get_$format: NUL character in line 2 of $kind file '$T/$kind'" \
        -R $format "$T/$kind" $R
done <<'EOF'
qrels qrels
qrels_jg qrels
qrels_prefs qrels
prefs prefs
EOF
expect_message 2 "get_results: NUL character in line 2 of results file '$T/results'" \
    $Q "$T/results"
expect_message 2 "get_results: NUL character in line 2 of results file '-'" \
    $Q - < "$T/results"
expect_message 2 "get_zscores: NUL character in line 2 of zscores file '$T/zscores'" \
    -Z "$T/zscores" $Q $R
