# Blank lines, lines of white space and comment lines, whose first
# non-blank character is '#', are skipped wherever they are in qrels,
# qrels_jg and prefs files. A file with no other lines is refused with
# exit 2. The comments have the fields of a data line, so -c would print
# a topic read from one.
. test/regress/lib.sh
R=test/regress/results

# skip NAME COMMENT: write test/regress/NAME to T with lines to skip
# added at the start, in the middle and at the end.
skip() {
    f=test/regress/$1
    { cat $f; echo "# $2"; } > "$T/$1.end"
    { echo "# $2"; echo '#'; cat $f; } > "$T/$1.start"
    { head -n 2 $f; echo; tail -n +3 $f; } > "$T/$1.blank"
    { head -n 2 $f; printf ' \t \n'; tail -n +3 $f; } > "$T/$1.space"
    { head -n 2 $f; echo "  # $2"; tail -n +3 $f; } > "$T/$1.indent"
    printf '# %s\n\n  # %s\n' "$2" "$2" > "$T/$1.only"
}
skip qrels 'x y 1'
skip prefs 'x y z 1'

# check FORMAT NAME ARGS...: the files T/NAME.* give the values of
# test/regress/NAME with -R FORMAT.
check() {
    fmt=$1 name=$2
    shift 2
    save_output "$T/plain" -c -q -R $fmt "$@" test/regress/$name $R
    for v in end start blank space indent; do
        expect_output "$T/plain" -c -q -R $fmt "$@" "$T/$name.$v" $R
    done
    expect_message 2 "No data lines in $name file '$T/$name.only'" \
        -R $fmt "$@" "$T/$name.only" $R
}

check qrels qrels -m num_rel
check qrels_jg qrels -m map_avgjg
check qrels_prefs qrels -m prefs_num_prefs_poss
check prefs prefs -m prefs_num_prefs_poss
