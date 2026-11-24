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

/* The product of two counts for a pool request, or -1, which the
   procedures refuse, if either count is negative or the product does not
   fit in a long, which has 32 bits on Windows */
long te_mul_counts(const long a, const long b)
{
    if (a < 0 || b < 0 || (a > 0 && b > LONG_MAX / a))
        return (-1);
    return (a * b);
}

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

/* Line numbers are held in a long, which has 32 bits on Windows. The
   reader-limits test lowers the limit. */
#ifndef TE_MAX_LINES
#define TE_MAX_LINES LONG_MAX
#endif

/* Prepare the size bytes of an input file, read into buf with room for
   two more: blank a UTF-8 byte-order mark, which the readers then skip as
   leading white space, end the last line with a newline, add a NUL, and
   return the number of lines. A NUL byte, a CR that does not end a line,
   or TE_MAX_LINES lines or more is an error: print a message naming the
   reader, the kind of file and the file, and return UNDEF. A CR that ends
   the file ends its last line. */
long te_prepare_lines(char *buf, size_t size, const char *reader,
                      const char *kind, const char *file)
{
    size_t i;
    long num_lines = 0;

    if (size >= 3 && 0 == memcmp(buf, "\357\273\277", 3))
        memset(buf, ' ', 3);
    if (size == 0 || buf[size - 1] != '\n')
        buf[size++] = '\n';
    buf[size] = '\0';

    for (i = 0; i < size; i++) {
        if (buf[i] == '\n') {
            if (++num_lines == TE_MAX_LINES) {
                fprintf(stderr, "trec_eval.%s: %ld lines or more "
                        "in %s file '%s'\n", reader, (long) TE_MAX_LINES,
                        kind, file);
                return (UNDEF);
            }
        } else if (buf[i] == '\0') {
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

/* On Windows, _read refuses a count above INT_MAX, and fread is not
   documented to split a larger request. The reader-limits test lowers
   the limit. */
#ifndef TE_READ_CHUNK
#define TE_READ_CHUNK ((size_t) 1 << 30)
#endif

/* Read up to size bytes of fd into buf, at most TE_READ_CHUNK bytes per
   request, and return the number read, which is less than size only at
   end of file or on an error */
size_t te_fread(char *buf, size_t size, FILE * fd)
{
    size_t done = 0;
    size_t request;
    size_t count;

    do {
        request = MIN(size - done, TE_READ_CHUNK);
        count = fread(buf + done, 1, request, fd);
        done += count;
    } while (count == request && done < size);
    return (done);
}

/* Read file into a new buffer *buf, prepared by te_prepare_lines, and
   return its number of lines, or print a message and return UNDEF */
long te_read_lines(const char *file, char **buf, const char *reader,
                   const char *kind)
{
    FILE *fd;
    long long file_size;
    size_t size;
    int read_ok;

    read_ok = NULL != (fd = fopen(file, "rb")) &&
        0 == fseek(fd, 0L, SEEK_END) && 0 < (file_size = ftell(fd)) &&
        (size = file_size) == file_size && size <= (size_t) -1 - 2 &&
        NULL != (*buf = malloc(size + 2)) &&
        0 == fseek(fd, 0L, SEEK_SET) && size == te_fread(*buf, size, fd);
    if (fd != NULL && 0 != fclose(fd))
        read_ok = 0;
    if (!read_ok) {
        fprintf(stderr, "trec_eval.%s: Cannot read %s file '%s'\n",
                reader, kind, file);
        return (UNDEF);
    }
    return (te_prepare_lines(*buf, size, reader, kind, file));
}

/* Skip the white space at *ptr. If the line is then a comment or blank,
   move *ptr to the next line, count the line in *line_num and return 1,
   warning of a blank line unless kind is NULL. Otherwise return 0. */
int te_skip_line(char **ptr, long *line_num, const char *kind,
                 const char *file)
{
    char *p = *ptr;

    while (*p != '\n' && isspace((unsigned char) *p))
        p++;
    *ptr = p;
    if (*p != '\n' && *p != '#')
        return (0);
    if (*p == '\n' && kind != NULL)
        fprintf(stderr, "Warning, blank line skipped "
                "(line %ld of %s file '%s').\n", *line_num + 1, kind, file);
    *ptr = index(p, '\n') + 1;
    (*line_num)++;
    return (1);
}
