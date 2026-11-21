# Malloc, Realloc and the pool helpers request n * sizeof(type) bytes.
# The macros return NULL when that does not fit in size_t. The pool
# helpers refuse negative requests and allocate at least one element.
# A stub allocator checks sizes without allocating large buffers.
. test/regress/lib.sh

./test_allocation-sizes ||
    fail "Malloc, Realloc or a pool helper requested a wrong size"
