# Helpers for the scripts in test/regress. Each script is run from the
# repository root as `sh test/regress/test_<name>.sh ./trec_eval`.
TREC_EVAL=${1:-./trec_eval}

fail() {
    echo "FAIL $0: $*" >&2
    exit 1
}

# expect_output FILE ARGS...: exit 0 and standard output must equal FILE.
expect_output() {
    expect_output_status 0 "$@"
}

# expect_output_status STATUS FILE ARGS...: check exit status and stdout.
expect_output_status() {
    want=$1 file=$2
    shift 2
    output=$(mktemp) || fail "mktemp failed"
    "$TREC_EVAL" "$@" > "$output" 2>/dev/null
    got=$?
    if ! diff - "$file" < "$output" >/dev/null; then
        rm -f "$output"
        fail "trec_eval $* differs from $file"
    fi
    rm -f "$output"
    [ "$got" = "$want" ] || fail "trec_eval $*: exit $got, expected $want"
}

# expect_status STATUS ARGS...: the exit status must equal STATUS.
expect_status() {
    want=$1
    shift
    "$TREC_EVAL" "$@" >/dev/null 2>&1
    got=$?
    [ "$got" = "$want" ] || fail "trec_eval $*: exit $got, expected $want"
}
