# Each judgments and zscores reader asks for a buffer of the whole file
# when its size does not fit in an int. A stub ftell reports the size of
# a small file as INT_MAX + 1, and a stub allocator refuses that size.
. test/regress/lib.sh

./test_file-sizes test/regress/qrels 2> "$T/error" ||
    fail "a reader did not ask for a buffer of the whole file"
