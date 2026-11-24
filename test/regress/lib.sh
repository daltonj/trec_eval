# Run from the repository root; temporary files live in T.
TREC_EVAL=${1:-./trec_eval}

fail() {
    echo "FAIL $0: $*" >&2
    exit 1
}

T=$(mktemp -d) || fail "mktemp -d failed"
trap 'rm -rf "$T"' EXIT

# long_bits: 64 if trec_eval accepts a long of 2^32 as -N, 32 if it
# refuses it as a build whose long has 32 bits does; any other result fails
# the script.
long_bits() {
    "$TREC_EVAL" -N 4294967296 -m num_q test/regress/qrels.one \
        test/regress/results.one > /dev/null 2> "$T/long-bits.error"
    case $? in
    0) echo 64 ;;
    1) grep -q "Invalid number of documents in collection" \
           "$T/long-bits.error" && echo 32 ;;
    esac
}

# Skip the rest of the script unless long has 64 bits.
need_long64() {
    case $(long_bits) in
    64) ;;
    32) echo "SKIP $0: needs a 64-bit long" >&2; exit 0 ;;
    *) fail "cannot tell the width of long: $(cat "$T/long-bits.error")" ;;
    esac
}

# Skip the rest of the script unless long has 32 bits.
need_long32() {
    case $(long_bits) in
    32) ;;
    64) echo "SKIP $0: needs a 32-bit long" >&2; exit 0 ;;
    *) fail "cannot tell the width of long: $(cat "$T/long-bits.error")" ;;
    esac
}

# expect_status STATUS ARGS...: run trec_eval ARGS, keeping its standard
# output and error, and check that it exits with STATUS.
expect_status() {
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

save_output() {
    te_file=$1
    shift
    expect_status 0 "$@"
    cp "$T/te.output" "$te_file"
}

expect_output() {
    te_file=$1
    shift
    expect_status 0 "$@"
    te_same output "$te_file"
}

# expect_result STATUS OUTPUT_FILE ERROR_FILE ARGS...
expect_result() {
    te_want=$1 te_output=$2 te_error=$3
    shift 3
    expect_status "$te_want" "$@"
    te_same output "$te_output"
    te_same error "$te_error"
}

expect_message() {
    te_want=$1 te_text=$2
    shift 2
    expect_status "$te_want" "$@"
    grep -F -q -e "$te_text" "$T/te.error" ||
        te_fail "no message containing: $te_text"
}
