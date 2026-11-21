# Run from the repository root; temporary files live in T.
TREC_EVAL=${1:-./trec_eval}

fail() {
    echo "FAIL $0: $*" >&2
    exit 1
}

T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

# te_run STATUS ARGS...: capture output and error, checking the exit status.
te_run() {
    te_want=$1
    shift
    te_args=$*
    "$TREC_EVAL" "$@" > "$T/te.output" 2> "$T/te.error"
    te_got=$?
    [ "$te_got" = "$te_want" ] || te_fail "exit $te_got, expected $te_want"
}

te_fail() {
    cat "$T/te.error" >&2
    fail "trec_eval $te_args: $*"
}

te_same() {
    diff "$T/te.$1" "$2" >&2 || te_fail "standard $1 differs from $2"
}

expect_status() {
    te_run "$@"
}

save_output() {
    te_file=$1
    shift
    te_run 0 "$@"
    cp "$T/te.output" "$te_file"
}

expect_output() {
    te_file=$1
    shift
    te_run 0 "$@"
    te_same output "$te_file"
}

# expect_result STATUS OUTPUT_FILE ERROR_FILE ARGS...
expect_result() {
    te_want=$1 te_output=$2 te_error=$3
    shift 3
    te_run "$te_want" "$@"
    te_same output "$te_output"
    te_same error "$te_error"
}

expect_message() {
    te_want=$1 te_text=$2
    shift 2
    te_run "$te_want" "$@"
    grep -F -q -e "$te_text" "$T/te.error" ||
        te_fail "no message containing: $te_text"
}
