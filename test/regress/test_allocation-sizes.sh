# Malloc and Realloc (common.h) request n * sizeof(type) bytes as a
# size_t, and return NULL without allocating when that does not fit.
# Judgments files of 2 GiB or more are impractical here, so a small
# program checks the byte counts the macros pass to malloc and realloc.
. test/regress/lib.sh

TEST_ALLOC="$T/alloc" make -s alloc-test || fail "cannot build the allocation test"
"$T/alloc" || fail "Malloc or Realloc requested a wrong size"
