/* 
   Copyright (c) 2008 - Chris Buckley. 

   Permission is granted for use and modification of this file for
   research, non-commercial purposes. 
*/

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"
#include "trec_format.h"

void *te_malloc_array(size_t n, size_t size)
{
    if (size > 0 && n > (size_t) -1 / size)
        return (NULL);
    /* malloc(0) may return NULL, which callers take for a failure */
    return (malloc(n * size > 0 ? n * size : 1));
}

void *te_realloc_array(void *ptr, size_t n, size_t size)
{
    if (size > 0 && n > (size_t) -1 / size)
        return (NULL);
    return (realloc(ptr, n * size));
}

/* The product of two counts for a pool request, or -1, which the
   procedures refuse, if either count is negative or the product does not
   fit in a long, which has 32 bits on Windows */
long te_mul_counts(const long a, const long b)
{
    if (a < 0 || b < 0 || (a > 0 && b > LONG_MAX / a))
        return (-1);
    return (a * b);
}

/* The space for a copy of string s, as a count for a pool request, or -1,
   which the procedures refuse, if it does not fit in a long */
long te_str_count(const char *s)
{
    size_t len = strlen(s);

    return (len < LONG_MAX ? (long) len + 1 : -1);
}

/* Utility malloc procedures for handling a malloc'd reusable object list that
   may need to be resized.
   Procedures take a current pointer, the current bound on the number of
   objects in the list, the needed number of objects on the list, and the
   size of an object.
   The procedures return the current pointer if the needed number is less
   than the current bound, otherwise they expand the space (either malloc
   or realloc depending on the procedure), resetting the current bound.
   It's an error if current_bound or needed is negative, or if the new
   bound does not fit in a long: NULL is returned.
   Procedures assume that if current bound is 0, the space has not been
   allocated at all yet.
*/

void *te_chk_and_malloc(void *ptr, long *current_bound,
                        const long needed, const size_t size)
{
    if (*current_bound < 0 || needed < 0)
        return (NULL);
    if (needed <= *current_bound && *current_bound > 0)
        return (ptr);
    if (needed > LONG_MAX - *current_bound)
        return (NULL);
    if (*current_bound > 0)
        Free(ptr);
    /* Allocate at least one element, so that a first request for
       nothing, such as an empty ranking, gets a buffer */
    *current_bound += needed > 0 ? needed : 1;
    return (te_malloc_array(*current_bound, size));
}

void *te_chk_and_realloc(void *ptr, long *current_bound,
                         const long needed, const int size)
{
    if (*current_bound < 0 || needed < 0)
        return (NULL);
    if (needed <= *current_bound && *current_bound > 0)
        return (ptr);
    if (needed > LONG_MAX - *current_bound)
        return (NULL);
    if (*current_bound == 0) {
        *current_bound += needed > 0 ? needed : 1;
        return (te_malloc_array(*current_bound, size));
    }
    *current_bound += needed;
    return (te_realloc_array(ptr, *current_bound, size));
}
