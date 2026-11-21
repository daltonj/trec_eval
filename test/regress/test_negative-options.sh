# Negative counts/levels fail before output; zero is a valid endpoint.
. test/regress/lib.sh
Q=test/regress/qrels
R=test/regress/results
while IFS='|' read -r option message; do
    printf "trec_eval: %s '-1'\n" "$message" > "$T/err"
    expect_result 1 /dev/null "$T/err" "$option" -1 $Q $R
done <<'EOF'
-N|Invalid number of documents in collection
-M|Invalid maximum number of documents per topic
-l|Unusable relevance level
EOF
expect_output test/regress/negative-max-docs/out.M0 \
    -N 0 -M 0 -l 0 -m num_ret -q $Q $R
