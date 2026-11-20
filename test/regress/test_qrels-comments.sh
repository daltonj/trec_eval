# Every '#' comment line in a qrels, qrels_jg or prefs file is skipped,
# wherever it is. A file holding only comments is refused with exit 2.
. test/regress/lib.sh
D=test/regress/qrels-comments

# check FORMAT NAME EXPECTED ARGS...: NAME is qrels or prefs.
check() {
    fmt=$1 name=$2 exp=$3
    shift 3
    for v in end two fields; do
        expect_output $exp -q -R $fmt "$@" $D/$name.$v $D/run
    done
    expect_message 2 "No data lines in $name file '$D/$name.only'" \
        -R $fmt "$@" $D/$name.only $D/run
}

check qrels qrels $D/out.num_rel -m num_rel
check qrels_jg qrels $D/out.map_avgjg -m map_avgjg
check qrels_prefs qrels $D/out.prefs -m prefs_num_prefs_poss
check prefs prefs $D/out.prefs -m prefs_num_prefs_poss
