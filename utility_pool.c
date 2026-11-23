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
    if (n > (size_t) -1 / size)
        return (NULL);
    /* malloc(0) may return NULL, which callers take for a failure */
    return (malloc(n > 0 ? n * size : 1));
}

void *te_realloc_array(void *ptr, size_t n, size_t size)
{
    if (n > (size_t) -1 / size)
        return (NULL);
    return (realloc(ptr, n * size));
}

/* Utility malloc procedures for handling a malloc'd reusable object list that
   may need to be resized.
   Procedures take a current pointer, the current bound on the number of
   objects in the list, the needed number of objects on the list, and the
   size of an object.
   The procedures return the current pointer if the needed number is less
   than the current bound, otherwise they expand the space (either malloc
   or realloc depending on the procedure), resetting the current bound.
   It's an error if current_bound or needed is negative, NULL is returned.
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
    if (*current_bound > 0)
        Free(ptr);
    /* Allocate at least one element, so that a first request for
       nothing, such as an empty ranking, gets a buffer */
    *current_bound += needed > 0 ? needed : 1;
    return ((void *) malloc(*current_bound * size));
}

void *te_chk_and_realloc(void *ptr, long *current_bound,
                         const long needed, const int size)
{
    if (*current_bound < 0 || needed < 0)
        return (NULL);
    if (needed <= *current_bound && *current_bound > 0)
        return (ptr);
    if (*current_bound == 0) {
        *current_bound += needed > 0 ? needed : 1;
        return ((void *) malloc(*current_bound * size));
    }
    *current_bound += needed;
    return ((void *) realloc(ptr, *current_bound * size));
}

/* Remove a UTF-8 byte-order mark before line termination is appended.
   At least one byte stays, since the judgments readers test buf[size - 1]. */
size_t te_skip_bom(char *buf, size_t size)
{
    if (size > 3 && 0 == memcmp(buf, "\357\273\277", 3)) {
        size -= 3;
        memmove(buf, buf + 3, size);
    }
    return (size);
}

/* Count the lines of an input file read into buf: size bytes that end in
   a newline, followed by a NUL. A NUL byte, or a CR that does not end a
   CRLF line ending, is an error: print a message naming the reader, the
   kind of file and the file, and return UNDEF. */
long te_count_lines(const char *buf, const size_t size, const char *reader,
                    const char *kind, const char *file)
{
    size_t i;
    long num_lines = 0;

    for (i = 0; i < size; i++) {
        if (buf[i] == '\n')
            num_lines++;
        else if (buf[i] == '\0') {
            fprintf(stderr, "trec_eval.%s: NUL character "
                    "in line %ld of %s file '%s'\n",
                    reader, num_lines + 1, kind, file);
            return (UNDEF);
        } else if (buf[i] == '\r' && buf[i + 1] != '\n') {
            fprintf(stderr, "trec_eval.%s: CR character "
                    "without LF in line %ld of %s file '%s'\n",
                    reader, num_lines + 1, kind, file);
            return (UNDEF);
        }
    }
    return (num_lines);
}
