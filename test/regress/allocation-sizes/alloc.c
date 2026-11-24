/* Malloc, Realloc and the pool helpers request n * sizeof(type) bytes.
   The macros return NULL when that does not fit in size_t. The pool
   helpers refuse negative requests and a bound beyond LONG_MAX, and
   allocate at least one element. te_mul_counts refuses a product beyond
   LONG_MAX.
   A stub allocator records requested sizes without allocating them. */
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
    return (record_malloc(size));
}

#define malloc record_malloc
#define realloc record_realloc
#include "common.h"
#include "utility_pool.c"

#define CHECK(expr) do { if (!(expr)) { puts(#expr); return (1); } } while (0)

int main(void)
{
    /* half * sizeof(short) overflows size_t by one. */
    size_t half = ((size_t) -1) / 2 + 1;
    long bound = 0;

    CHECK(NULL == te_chk_and_malloc(buffer, &bound, -1, sizeof(long)) && bound == 0);
    CHECK(NULL == te_chk_and_realloc(buffer, &bound, -1, sizeof(long)) && bound == 0);
    CHECK(NULL != te_chk_and_malloc(NULL, &bound, 0, sizeof(long)) &&
          bound == 1 && requested == sizeof(long));
    bound = 0;
    CHECK(NULL != te_chk_and_realloc(NULL, &bound, 0, sizeof(long)) &&
          bound == 1 && requested == sizeof(long));
    CHECK(te_mul_counts(LONG_MAX / 2, 2) == LONG_MAX - 1);
    CHECK(te_mul_counts(LONG_MAX / 2 + 1, 2) == -1);
    CHECK(te_mul_counts(0, LONG_MAX) == 0 && te_mul_counts(LONG_MAX, 0) == 0);
    CHECK(te_mul_counts(-1, 0) == -1 && te_mul_counts(0, -1) == -1);
    /* A request within a large bound is met; one that would take the
       bound beyond LONG_MAX is refused before the old space is freed */
    bound = LONG_MAX / 2 + 2;
    CHECK(buffer == te_chk_and_malloc(buffer, &bound, LONG_MAX / 2 + 1, 1));
    CHECK(buffer == te_chk_and_realloc(buffer, &bound, LONG_MAX / 2 + 1, 1));
    CHECK(NULL == te_chk_and_malloc(buffer, &bound, LONG_MAX / 2 + 3, 1) &&
          bound == LONG_MAX / 2 + 2);
    CHECK(NULL == te_chk_and_realloc(buffer, &bound, LONG_MAX / 2 + 3, 1) &&
          bound == LONG_MAX / 2 + 2);
    /* setup_gains asks for no gains for a topic whose grades are all
       negative */
    CHECK(NULL != Malloc(0, long) && requested == 1);
    requested = 0;
    CHECK(NULL == Malloc(half, short) && requested == 0);
    CHECK(NULL == Realloc(buffer, half, short) && requested == 0);
    CHECK(NULL != Malloc(16, long) && requested == 16 * sizeof(long));
    CHECK(NULL != Realloc(buffer, 32, long) && requested == 32 * sizeof(long));
#if SIZE_MAX > UINT_MAX
    CHECK(NULL != Malloc((size_t) UINT_MAX + 1, char) &&
          requested == (size_t) UINT_MAX + 1);
    CHECK(NULL != Realloc(buffer, (size_t) UINT_MAX + 1, char) &&
          requested == (size_t) UINT_MAX + 1);
#endif
    return (0);
}
