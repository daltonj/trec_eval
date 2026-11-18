# Count qid comparisons to keep topic matching linear in the input size.
. test/regress/lib.sh
D=test/regress/topic-lookup
T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

cat > "$T/Makefile" <<'MAKE'
include Makefile
lookup-test:
	$(CC) $(CFLAGS) -o "$(LOOKUP_BINARY)" $(filter-out trec_eval.c,$(SRCS)) test/regress/topic-lookup/lookup.c -lm
MAKE
make -s -f "$T/Makefile" lookup-test LOOKUP_BINARY="$T/trec_eval" \
    >"$T/build.out" 2>&1 || fail "cannot build topic lookup counter"
awk 'BEGIN { for (i = 1; i <= 100; i++) print i, "0 d 1" }' > "$T/qrels"
awk 'BEGIN { for (i = 1; i <= 100; i++) print i, "Q0 d 1 1 r" }' > "$T/results"
"$T/trec_eval" -m num_ret "$T/qrels" "$T/results" \
    > "$T/out" 2> "$T/comparisons" || fail "topic evaluation failed"
awk '$1 == "num_ret" && $2 == "all" && $3 == 100 { ok = 1 }
     END { exit !ok }' "$T/out" || fail "wrong num_ret"
comparisons=$(cat "$T/comparisons")
[ "$comparisons" -le 1000 ] || fail "$comparisons topic comparisons for 100 topics"
