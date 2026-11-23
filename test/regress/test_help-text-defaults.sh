# The help text gives the default of -N as 0, and the long form of -M
# with two dashes.
. test/regress/lib.sh
save_output "$T/help" -h
grep -F -x ' -N <num>: Number of docs in collection. Default is 0.' \
    "$T/help" >/dev/null || fail "help text for -N"
grep -F -x ' --Max_retrieved_per_topic num:' \
    "$T/help" >/dev/null || fail "help text for --Max_retrieved_per_topic"
