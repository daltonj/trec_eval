# Helpers for the scripts in test/regress. Each script is run from the
# repository root as `sh test/regress/test_<name>.sh ./trec_eval`, and may
# keep temporary files in the directory T. Each expect_ check runs
# trec_eval once; on failure it shows the difference and what trec_eval
# wrote to standard error.
TREC_EVAL=${1:-./trec_eval}

fail() {
    echo "FAIL $0: $*" >&2
    exit 1
}

T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

# te_run STATUS ARGS...: run trec_eval with ARGS, keeping its standard
# output and error in T, and fail unless it exits with STATUS.
te_run() {
    te_want=$1
    shift
    "$TREC_EVAL" "$@" > "$T/te.out" 2> "$T/te.err"
    te_got=$?
    [ "$te_got" = "$te_want" ] ||
        te_fail "trec_eval $*: exit $te_got, expected $te_want"
}

# te_fail MESSAGE: show the standard error of the last run, and fail.
te_fail() {
    cat "$T/te.err" >&2
    fail "$@"
}

# expect_status STATUS ARGS...: exit STATUS.
expect_status() {
    te_run "$@"
}

# expect_output FILE ARGS...: exit 0, and standard output equal to FILE.
expect_output() {
    expect_output_status 0 "$@"
}

# expect_output_status STATUS FILE ARGS...: exit STATUS, and standard
# output equal to FILE.
expect_output_status() {
    te_want=$1 te_file=$2
    shift 2
    te_run "$te_want" "$@"
    diff "$T/te.out" "$te_file" >&2 ||
        te_fail "trec_eval $*: standard output differs from $te_file"
}

# expect_stderr STATUS FILE ARGS...: exit STATUS, and standard error
# equal to FILE.
expect_stderr() {
    te_want=$1 te_file=$2
    shift 2
    te_run "$te_want" "$@"
    diff "$T/te.err" "$te_file" >&2 ||
        fail "trec_eval $*: standard error differs from $te_file"
}

# expect_message STATUS TEXT ARGS...: exit STATUS, and TEXT in a line of
# standard error.
expect_message() {
    te_want=$1 te_text=$2
    shift 2
    te_run "$te_want" "$@"
    grep -F -q -e "$te_text" "$T/te.err" ||
        te_fail "trec_eval $*: no message containing: $te_text"
}
