# Output checks require the expected status and exact stdout from one run.
. test/regress/lib.sh
expect_output test/out.test test/qrels.test test/results.test

T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT
printf 'expected\n' > "$T/expected"
printf 'different\n' > "$T/different"
cat > "$T/eval" <<'EOF'
#!/bin/sh
echo call >> "$TEST_CALLS"
printf 'expected\n'
exit "$TEST_STATUS"
EOF
chmod +x "$T/eval" || fail "cannot make test evaluator executable"
TREC_EVAL=$T/eval
TEST_CALLS=$T/calls
export TEST_CALLS TEST_STATUS

# Each assertion must run the evaluator once, including failed assertions.
check() {
    want=$1
    shift
    : > "$TEST_CALLS"
    ( "$@" ) > "$T/output" 2>&1
    got=$?
    [ "$got" -eq "$want" ] || fail "$*: exit $got, expected $want"
    [ "$(wc -l < "$TEST_CALLS")" -eq 1 ] || fail "$*: did not run once"
}

TEST_STATUS=0
check 0 expect_output "$T/expected"
check 1 expect_output "$T/different"
TEST_STATUS=97
check 1 expect_output "$T/expected"
check 1 expect_output "$T/different"
check 0 expect_output_status 97 "$T/expected"
check 1 expect_output_status 97 "$T/different"
check 1 expect_output_status 2 "$T/expected"
TEST_STATUS=0
check 1 expect_output_status 97 "$T/expected"
