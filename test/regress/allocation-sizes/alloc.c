/* Checks the byte counts requested by Malloc, Realloc and the pool
   helpers, using stubs that record the count without allocating. */
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
    long bound = 0;

    check(NULL == te_chk_and_malloc(buffer, &bound, -1, sizeof(long)) &&
          bound == 0,
          "te_chk_and_malloc accepted a negative first request");
    bound = 0;
    check(NULL == te_chk_and_realloc(buffer, &bound, -1, sizeof(long)) &&
          bound == 0,
          "te_chk_and_realloc accepted a negative first request");

    bound = 0;
    check(NULL != te_chk_and_malloc(NULL, &bound, 0, sizeof(long)) &&
          bound == 1 && requested == sizeof(long),
          "te_chk_and_malloc did not allocate one element for a zero request");
    bound = 0;
    check(NULL != te_chk_and_realloc(NULL, &bound, 0, sizeof(long)) &&
          bound == 1 && requested == sizeof(long),
          "te_chk_and_realloc did not allocate one element for a zero request");

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
