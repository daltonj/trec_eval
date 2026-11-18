#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static long comparisons;

static int count_strcmp(const char *left, const char *right)
{
    comparisons++;
    return strcmp(left, right);
}

static void print_comparisons(void)
{
    fprintf(stderr, "%ld\n", comparisons);
}

#define main trec_eval_main
#define strcmp count_strcmp
#include "../../../trec_eval.c"
#undef strcmp
#undef main

int main(int argc, char **argv)
{
    atexit(print_comparisons);
    return trec_eval_main(argc, argv);
}
