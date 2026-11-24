/* Reading of the input files, shared by the judgments, results and zscores
   readers */

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"
#include "trec_format.h"

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
