# Both preference rank comparators must preserve the sign of differences
# beyond the int range, with the judgment group and relevance held equal.
. test/regress/lib.sh

TEST_PREFS_RANK="$T/cmp" make -s prefs-rank-test ||
    fail "cannot build the preference rank comparator test"
"$T/cmp" || fail "preference ranks compared in the wrong order"
