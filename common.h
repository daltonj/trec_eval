/* 
   Copyright (c) 2008 - Chris Buckley. 

   Permission is granted for use and modification of this file for
   research, non-commercial purposes. 
*/
#ifndef COMMONH
#define COMMONH

#include <stdio.h>

#ifndef FALSE
#define FALSE   0
#endif
#ifndef TRUE
#define TRUE    1
#endif

#define UNDEF   -1

#define MAX(A,B)  ((A) > (B) ? (A) : (B))
#define MIN(A,B)  ((A) > (B) ? (B) : (A))

#ifndef MAXLONG
#define MAXLONG 2147483647L     /* largest long int. no. */
#endif

/*
 * Some useful macros for making malloc et al easier to use.
 * Macros handle the casting and the like that's needed.
 * Malloc and Realloc return NULL, without allocating, if n * sizeof(type)
 * does not fit in a size_t.
 */
void *te_malloc_array(size_t n, size_t size);
void *te_realloc_array(void *ptr, size_t n, size_t size);
#define Malloc(n,type) (type *) te_malloc_array((n), sizeof(type))
#define Realloc(loc,n,type) (type *) te_realloc_array((loc), (n), sizeof(type))
#define Free(loc) (void) free( (char *)(loc) )

#endif                          /* COMMONH */
