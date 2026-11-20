# The qrels_prefs and prefs readers refuse a relevance level that is not
# a number, or is NaN, with a message naming the line and the file. A
# level with characters after the number gives a warning, and the number
# is used. The .1 files give d1 the level 1; the .1x files must give the
# same values.
. test/regress/lib.sh
D=test/regress/prefs-level-validation
R=test/regress/results
m="-m prefs_num_prefs_poss -m prefs_simp"

# check FORMAT PREFIX: read the files PREFIX.* with -R FORMAT
check() {
    for v in abc nan; do
        expect_message 2 "(line 1 of $2 file '$D/$2.$v')." \
            -R $1 $m $D/$2.$v $R
    done
    expect_output $D/out.grade1 -R $1 $m $D/$2.1 $R
    expect_output $D/out.grade1 -R $1 $m $D/$2.1x $R
    expect_message 0 "value \`1x' partially read as \`1' (line 1 of $2 file" \
        -R $1 $m $D/$2.1x $R
}
check qrels_prefs qrels
check prefs prefs
