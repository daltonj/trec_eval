# Both preference rank comparators must preserve the sign of differences
# beyond the int range, with the judgment group and relevance held equal.
. test/regress/lib.sh

./test_prefs-rank-comparators ||
    fail "preference ranks compared in the wrong order"
