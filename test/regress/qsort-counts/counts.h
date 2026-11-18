#include <stdlib.h>

/* Exercise the real call sites without allocating billions of records. */
#define qsort(base, count, width, compare) \
    ((void) sizeof(char[_Generic((count), int: -1, default: 1)]), \
     qsort(base, count, width, compare))
