/*
   Copyright (c) 2008 - Chris Buckley.

   Permission is granted for use and modification of this file for
   research, non-commercial purposes.
*/

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "trec_format.h"
#include "functions.h"
#include <ctype.h>
#include <errno.h>
#include <stdarg.h>

/*
 * End an error or warning from a string conversion. Unless fmt is NULL,
 * fmt and the arguments after it, formatted as by printf, follow in
 * parentheses: they say where the value comes from.
 */
static void end_message(const char * fmt, va_list ap) {
    if (fmt != NULL) {
        fputs(" (", stderr);
        vfprintf(stderr, fmt, ap);
        fputc(')', stderr);
    }
    fputs(".\n", stderr);
}

/*
 * Convert a string to a long, with error checking.
 */
long str_to_long_ctx(const char * str, int * error, const char * fmt, ...) {
    char *endptr = NULL;
    va_list ap;
    errno = 0;

    *error = 0;
    long value = strtol(str, &endptr, 10);
    if (errno == ERANGE) {
        /* value is 0 */
        fprintf(stderr, "Error: value `%s' out of range", str);
        *error = 1;
    } else if (endptr == str) {
        /* value is 0 */
        fprintf(stderr, "Error: value `%s' could not be parsed", str);
        *error = 1;
    } else if (*endptr != '\0') {
        fprintf(stderr, "Warning, value `%s' partially read as `%ld'", str, value);
    } else {
        return value;
    }
    va_start(ap, fmt);
    end_message(fmt, ap);
    va_end(ap);
    return value;
}

/*
 * Convert a string to a double, with the error checking of
 * str_to_long_ctx. NaN is an error. Overflow to infinity and underflow
 * to zero produce warnings; finite subnormal results are accepted
 * without a range warning. Characters after the number give a warning.
 */
double str_to_double_ctx(const char * str, int * error,
                         const char * fmt, ...) {
    char *end = NULL;
    va_list ap;
    errno = 0;

    *error = 0;
    double value = strtod(str, &end);
    if (end == str) {
        fprintf(stderr, "Error: value `%s' could not be parsed", str);
        *error = 1;
    } else if (isnan(value)) {
        fprintf(stderr, "Error: value `%s' is not a number", str);
        *error = 1;
    } else if (errno == ERANGE && (isinf(value) || value == 0.0)) {
        fprintf(stderr, "Warning, value `%s' out of range, read as `%g'",
                str, value);
    } else if (*end != '\0') {
        fprintf(stderr, "Warning, value `%s' partially read as `%.*s'",
                str, (int) (end - str), str);
    } else {
        return value;
    }
    va_start(ap, fmt);
    end_message(fmt, ap);
    va_end(ap);
    return value;
}

long str_to_long(const char * str, int * error) {
    return str_to_long_ctx(str, error, NULL);
}

/* Read all relevance information from text_qrels_file.
Relevance for each docno to qid is determined from text_qrels_file, which
consists of text tuples of the form
   qid  iter  docno  rel
giving TREC document numbers (docno, a string) and their relevance (rel,
an integer between -127 and 127) to query qid (a string).  Iter is ignored
Fields are separated by whitespace, string fields can contain no whitespace.
File may contain no NULL characters.

All <docno,rel> pairs stored in per query arrays within all_rel_info.
Each list of query judgments is sorted lexicographically by docno.
*/

/* Declarations in trec_eval.h (generic q info) and trec_format.h
   (qrels specific info) for the output format */
/*
    typedef struct {                    * For each relevance judgement *
        char *docno;                       * document id *
        long rel;                          * document judgement *
    } TEXT_QRELS;

    typedef struct {                    * For each qid in query *
        long num_text_qrels;               * number of judged documents *
        TEXT_QRELS *text_qrels;            * Array of judged TEXT_QRELS.
                          Kept sorted by docno *
    } TEXT_QRELS_INFO;

    typedef struct {
        char *qid;                      * query id *
        char *rel_format;               * format of rel_info data. Eg, "qrels" *
        void *q_rel_info;               * relevance info for this qid *
    } REL_INFO;

    typedef struct {                    * Overall relevance judgements *
        long num_q_rels;                * Number of REL_INFO queries *
        long max_num_q_rels;            * Num queries space reserved for *
        REL_INFO *rel_info;             * Array of REL_INFO queries *
    } ALL_REL_INFO;
*/
/* Temp structure for values in input line */
typedef struct
{
    char *qid;
    char *docno;
    long rel;
} LINES;

static int parse_qrels_line(char **start_ptr, char **qid_ptr,
                            char **docno_ptr, char **rel_ptr);

static int comp_lines_qid_docno(const void * ptr1, const void * ptr2);

/* static pools of memory, allocated here and never changed.
   Declared static so one day I can write a cleanup procedure to free them */
static char *trec_qrels_buf = NULL;
static TEXT_QRELS_INFO *text_info_pool = NULL;
static TEXT_QRELS *text_qrels_pool = NULL;
static REL_INFO *rel_info_pool = NULL;

int te_get_qrels(EPI *epi, char *text_qrels_file, ALL_REL_INFO *all_rel_info)
{
    FILE *fd;
    long long file_size;
    size_t size = 0;
    char *ptr;
    char *current_qid;
    char *rel_ptr;
    long i;
    LINES *lines;
    LINES *line_ptr;
    long num_lines;
    long line_num = 0;
    long num_qid;
    int error = 0;
    /* current pointers into static pools above */
    REL_INFO *rel_info_ptr;
    TEXT_QRELS_INFO *text_info_ptr;
    TEXT_QRELS *text_qrels_ptr;

    /* Read entire file into memory */
    if (!(fd = fopen(text_qrels_file, "rb")) ||
        fseek(fd, 0L, SEEK_END) != 0 || 0 >= (file_size = ftell(fd)) ||
        (size = file_size) != file_size ||
        NULL == (trec_qrels_buf = malloc(size + 2)) ||
        -1 == fseek(fd, 0L, SEEK_SET) ||
        size != fread(trec_qrels_buf, 1, size, fd) ||
        -1 == fclose(fd))
    {
        fprintf(stderr,
                "trec_eval.get_qrels: Cannot read qrels file '%s'\n",
                text_qrels_file);
        return (UNDEF);
    }

    size = te_skip_bom(trec_qrels_buf, size);

    /* Append ending newline if not present, Append NULL terminator */
    if (trec_qrels_buf[size - 1] != '\n')
    {
        trec_qrels_buf[size] = '\n';
        size++;
    }
    trec_qrels_buf[size] = '\0';

    /* Count number of lines in file */
    if (UNDEF == (num_lines = te_count_lines(trec_qrels_buf, size,
                                             "get_qrels", "qrels",
                                             text_qrels_file)))
        return (UNDEF);

    /* Get all lines */
    if (NULL == (lines = Malloc(num_lines, LINES)))
        return (UNDEF);
    line_ptr = lines;
    ptr = trec_qrels_buf;
    while (*ptr) {
		/* Skip comment lines, and blank lines with a warning */
		while (*ptr != '\n' && isspace(*ptr))
			ptr++;
		if (*ptr == '\n' || *ptr == '#') {
			if (*ptr == '\n')
				fprintf(stderr, "Warning, blank line skipped "
						"(line %ld of qrels file '%s').\n",
						line_num + 1, text_qrels_file);
			ptr = index(ptr, '\n') + 1;
			line_num++;
			continue;
		}
		if (UNDEF == parse_qrels_line (&ptr, &line_ptr->qid,
									   &line_ptr->docno, &rel_ptr)) {
			fprintf (stderr, "trec_eval.get_qrels: Malformed line %ld\n",
					 line_num + 1);
			return (UNDEF);
		}
		line_ptr->rel = str_to_long_ctx(rel_ptr, &error,
										"document '%s' of topic '%s' in "
										"line %ld of qrels file '%s'",
										line_ptr->docno, line_ptr->qid,
										line_num + 1, text_qrels_file);
		if (error) {
			fprintf(stderr, "trec_eval: Unusable relevance level '%s'\n",
					rel_ptr);
			return (UNDEF);
		}
		if (line_ptr->rel < MIN_REL_LEVEL || line_ptr->rel > MAX_REL_LEVEL) {
			fprintf(stderr, "trec_eval.get_qrels: Relevance level '%s' "
					"for document '%s' of topic '%s' in "
					"line %ld of qrels file '%s' is outside %d to %d\n",
					rel_ptr, line_ptr->docno, line_ptr->qid,
					line_num + 1, text_qrels_file,
					MIN_REL_LEVEL, MAX_REL_LEVEL);
			return (UNDEF);
		}
		line_ptr++;
		line_num++;
    }
    num_lines = line_ptr - lines;
    if (num_lines == 0) {
        fprintf(stderr,
                "trec_eval.get_qrels: No data lines in qrels file '%s'\n",
                text_qrels_file);
        return (UNDEF);
    }

    /* Sort all lines by qid, then docno */
    qsort((char *)lines, num_lines, sizeof(LINES), comp_lines_qid_docno);

    /* Go through lines and count number of qid */
    num_qid = 1;
    for (i = 1; i < num_lines; i++)
    {
        if (strcmp(lines[i - 1].qid, lines[i].qid))
            /* New query */
            num_qid++;
    }

    /* Allocate space for queries */
    if (NULL == (rel_info_pool = Malloc(num_qid, REL_INFO)) ||
        NULL == (text_info_pool = Malloc(num_qid, TEXT_QRELS_INFO)) ||
        NULL == (text_qrels_pool = Malloc(num_lines, TEXT_QRELS)))
        return (UNDEF);

    rel_info_ptr = rel_info_pool;
    text_info_ptr = text_info_pool;
    text_qrels_ptr = text_qrels_pool;

    /* Go through lines and store all info */
    current_qid = "";
    for (i = 0; i < num_lines; i++)
    {
        if (strcmp(current_qid, lines[i].qid))
        {
            /* New query.  End old query and start new one */
            if (i != 0)
            {
                text_info_ptr->num_text_qrels =
                    text_qrels_ptr - text_info_ptr->text_qrels;
                text_info_ptr++;
                rel_info_ptr++;
            }
            current_qid = lines[i].qid;
            text_info_ptr->text_qrels = text_qrels_ptr;
            *rel_info_ptr = (REL_INFO){
                current_qid, "qrels", text_info_ptr};
        }
        text_qrels_ptr->docno = lines[i].docno;
        text_qrels_ptr->rel = lines[i].rel;
        text_qrels_ptr++;
    }
    /* End last qid */
    text_info_ptr->num_text_qrels = text_qrels_ptr - text_info_ptr->text_qrels;

    all_rel_info->num_q_rels = num_qid;
    all_rel_info->rel_info = rel_info_pool;

    Free(lines);
    return (1);
}

static int comp_lines_qid_docno(const void * ptr1, const void * ptr2)
{
    LINES* p1 = (LINES *) ptr1;
    LINES* p2 = (LINES *) ptr2;
    int cmp = strcmp(p1->qid, p2->qid);
    if (cmp)
        return (cmp);
    return (strcmp(p1->docno, p2->docno));
}

static int
parse_qrels_line(char **start_ptr, char **qid_ptr,
                 char **docno_ptr, char **rel_ptr)
{
    char *ptr = *start_ptr;

    /* Get qid */
    while (*ptr != '\n' && isspace(*ptr))
        ptr++;
    *qid_ptr = ptr;
    while (!isspace(*ptr))
        ptr++;
    if (*ptr == '\n')
        return (UNDEF);
    *ptr++ = '\0';
    /* Get iter, ignore */
    while (*ptr != '\n' && isspace(*ptr))
        ptr++;
    while (!isspace(*ptr))
        ptr++;
    if (*ptr == '\n')
        return (UNDEF);
    ptr++;
    /* Get docno */
    while (*ptr != '\n' && isspace(*ptr))
        ptr++;
    *docno_ptr = ptr;
    while (!isspace(*ptr))
        ptr++;
    if (*ptr == '\n')
        return (UNDEF);
    *ptr++ = '\0';
    /* Get relevance */
    while (*ptr != '\n' && isspace(*ptr))
        ptr++;
    if (*ptr == '\n')
        return (UNDEF);
    *rel_ptr = ptr;
    while (!isspace(*ptr))
        ptr++;
    if (*ptr != '\n')
    {
        *ptr++ = '\0';
        while (*ptr != '\n' && isspace(*ptr))
            ptr++;
        if (*ptr != '\n')
            return (UNDEF);
    }
    *ptr++ = '\0';
    *start_ptr = ptr;
    return (0);
}

int te_get_qrels_cleanup()
{
    if (trec_qrels_buf != NULL)
    {
        Free(trec_qrels_buf);
        trec_qrels_buf = NULL;
    }
    if (text_info_pool != NULL)
    {
        Free(text_info_pool);
        text_info_pool = NULL;
    }
    if (text_qrels_pool != NULL)
    {
        Free(text_qrels_pool);
        text_qrels_pool = NULL;
    }
    if (rel_info_pool != NULL)
    {
        Free(rel_info_pool);
        rel_info_pool = NULL;
    }
    return (1);
}
