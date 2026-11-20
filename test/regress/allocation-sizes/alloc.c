/* Checks the byte counts that Malloc and Realloc in common.h pass to
   malloc and realloc, through stubs that record the count without
   allocating. test_allocation-sizes.sh builds it with "make alloc-test";
   the exit status is 0 if every check holds. */
#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static char buffer[1];
static size_t requested;

static void *record_malloc(size_t size)
{
    requested = size;
    return (buffer);
}

static void *record_realloc(void *ptr, size_t size)
{
    (void) ptr;
    requested = size;
    return (buffer);
}

#define malloc record_malloc
#define realloc record_realloc
#include "common.h"
#include "utility_pool.c"

static int failed = 0;

static void check(int ok, const char *call)
{
    if (!ok) {
        printf("%s\n", call);
        failed = 1;
    }
}

int main(void)
{
    /* half * sizeof(short) is one more than the largest size_t */
    size_t half = ((size_t) -1) / 2 + 1;
    size_t n = 1;

    check(NULL != Malloc(n++, char) && n == 2 && requested == sizeof(char),
          "Malloc(n++, char) did not evaluate n once and request one char");
    n = 1;
    requested = 0;
    check(NULL != Realloc(buffer, n++, char) && n == 2 &&
          requested == sizeof(char),
          "Realloc(ptr, n++, char) did not evaluate n once and request one char");

    requested = 0;
    check(NULL == Malloc(half, short) && requested == 0,
          "Malloc(SIZE_MAX / 2 + 1, short) allocated");
    check(NULL == Malloc(-1, long) && requested == 0,
          "Malloc(-1, long) allocated");
    check(NULL == Realloc(buffer, half, short) && requested == 0,
          "Realloc(ptr, SIZE_MAX / 2 + 1, short) allocated");

    check(NULL != Malloc(16, long) && requested == 16 * sizeof(long),
          "Malloc(16, long) did not request 16 longs");
    check(NULL != Realloc(buffer, 32, long) && requested == 32 * sizeof(long),
          "Realloc(ptr, 32, long) did not request 32 longs");
#if SIZE_MAX > UINT_MAX
    check(NULL != Malloc((size_t) UINT_MAX + 1, char) &&
          requested == (size_t) UINT_MAX + 1,
          "Malloc(UINT_MAX + 1, char) did not request UINT_MAX + 1 bytes");
    check(NULL != Realloc(buffer, (size_t) UINT_MAX + 1, char) &&
          requested == (size_t) UINT_MAX + 1,
          "Realloc(ptr, UINT_MAX + 1, char) did not request UINT_MAX + 1 bytes");
#endif
    return (failed);
}
