# Blank lines, lines of only whitespace, and lines whose first non-blank
# character is '#' are skipped in qrels, qrels_jg and prefs files, as in
# results files. A file with no other lines is refused with exit 2.
. test/regress/lib.sh
D=test/regress/qrels-blank-lines

# The whitespace-only lines are written here, so that the fixtures have
# no trailing whitespace.
printf '1 0 d1 1\n1 0 d2 0\n \t \n1 0 d3 2\n2 0 d1 0\n2 0 d2 1\n' \
    > "$T/qrels.space"
printf '1 A 0 d1 1\n1 A 0 d2 0\n \t \n1 A 0 d3 2\n2 A 0 d1 0\n2 A 0 d2 1\n' \
    > "$T/prefs.space"

# check FORMAT NAME EXPECTED ARGS...: NAME is qrels or prefs. With -c, a
# topic read from an indented comment would appear in the output.
check() {
    fmt=$1 name=$2 exp=$3
    shift 3
    for judg in $D/$name.blank "$T/$name.space" $D/$name.indent; do
        expect_output $exp -c -q -R $fmt "$@" "$judg" $D/run
    done
    expect_message 2 "No data lines in $name file '$D/$name.only'" \
        -R $fmt "$@" $D/$name.only $D/run
}

check qrels qrels $D/out.num_rel -m num_rel
check qrels_jg qrels $D/out.map_avgjg -m map_avgjg
check qrels_prefs qrels $D/out.prefs -m prefs_num_prefs_poss
check prefs prefs $D/out.prefs -m prefs_num_prefs_poss
