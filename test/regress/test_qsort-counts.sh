# A signed int cast can turn a long record count above INT_MAX into a
# negative value, which qsort converts to a much larger size_t.
. test/regress/lib.sh
make -s qsort-counts-test || fail "cannot check qsort counts"
