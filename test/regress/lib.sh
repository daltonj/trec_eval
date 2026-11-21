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
    te_args=$*
    "$TREC_EVAL" "$@" > "$T/te.output" 2> "$T/te.error"
    te_got=$?
    [ "$te_got" = "$te_want" ] || te_fail "exit $te_got, expected $te_want"
}

# te_fail MESSAGE: show the standard error of the last run, and fail.
te_fail() {
    cat "$T/te.error" >&2
    fail "trec_eval $te_args: $*"
}

# te_same STREAM FILE: fail unless the standard STREAM, output or error,
# of the last run equals FILE.
te_same() {
    diff "$T/te.$1" "$2" >&2 || te_fail "standard $1 differs from $2"
}

# expect_status STATUS ARGS...: exit STATUS.
expect_status() {
    te_run "$@"
}

# save_output FILE ARGS...: exit 0, and write standard output to FILE.
save_output() {
    te_file=$1
    shift
    te_run 0 "$@"
    cp "$T/te.output" "$te_file"
}

# expect_output FILE ARGS...: exit 0, and standard output equal to FILE.
expect_output() {
    te_file=$1
    shift
    te_run 0 "$@"
    te_same output "$te_file"
}

# expect_result STATUS OUTPUT ERROR ARGS...: exit STATUS, standard output
# equal to the file OUTPUT, and standard error equal to the file ERROR.
expect_result() {
    te_want=$1 te_output=$2 te_error=$3
    shift 3
    te_run "$te_want" "$@"
    te_same output "$te_output"
    te_same error "$te_error"
}

# expect_message STATUS TEXT ARGS...: exit STATUS, and TEXT in a line of
# standard error.
expect_message() {
    te_want=$1 te_text=$2
    shift 2
    te_run "$te_want" "$@"
    grep -F -q -e "$te_text" "$T/te.error" ||
        te_fail "no message containing: $te_text"
}
