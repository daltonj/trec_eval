/* 
   Copyright (c) 2008 - Chris Buckley. 

   Permission is granted for use and modification of this file for
   research, non-commercial purposes. 
*/

#include "common.h"
#include "sysfunc.h"
#include "trec_eval.h"
#include "functions.h"

/* Procedures for initializing gain values from command-line parameters */

static int valid_int(const char *str)
{
    // Handle negative numbers.
    //
    if (*str == '-' || *str == '+')
        ++str;

    // Handle empty string or just "-" or "+".
    //
    if (!*str)
        return 0;

    // Check for non-digit chars in the rest of the stirng.
    //
    while (*str) {
        if (!isdigit(*str))
            return 0;
        else
            ++str;
    }

    return 1;
}

/* Each of the pairs of measure tm names a different relevance level */
static int check_gain_params(const TREC_MEAS * tm,
                             const DOUBLE_PARAM_PAIR * pairs, long num_pairs)
{
    long *levels;
    long i, j;
    int error = 0;

    if (NULL == (levels = Malloc(num_pairs, long)))
        return (UNDEF);
    for (i = 0; i < num_pairs; i++) {
        if (valid_int(pairs[i].name))
            levels[i] = str_to_long_ctx(pairs[i].name, &error,
                                        "relevance level of measure '%s'",
                                        tm->name);
        else
            error = 1;
        if (error) {
            fprintf(stderr,
                    "trec_eval: Unusable relevance level '%s' of measure '%s'\n",
                    pairs[i].name, tm->name);
            Free(levels);
            return (UNDEF);
        }
        for (j = 0; j < i; j++) {
            if (levels[j] == levels[i]) {
                fprintf(stderr,
                        "trec_eval: relevance level %ld of measure '%s' is given more than once\n",
                        levels[i], tm->name);
                Free(levels);
                return (UNDEF);
            }
        }
    }
    Free(levels);
    return (1);
}

/* Measure is a single double with gains for relevance levels as
   name=double params */
int te_init_meas_s_double_p_gain(EPI * epi, TREC_MEAS * tm, TREC_EVAL * eval)
{
    if (UNDEF == te_init_meas_s_double_p_pair(epi, tm, eval))
        return (UNDEF);
    return (check_gain_params(tm, tm->meas_params->param_values,
                              tm->meas_params->num_params));
}

/* rbp and rbp_resid: the persistence p, at least 0 and less than 1, and
   gains for relevance levels, which rbp_resid accepts and ignores */
int te_init_meas_s_double_p_rbp(EPI * epi, TREC_MEAS * tm, TREC_EVAL * eval)
{
    DOUBLE_PARAM_PAIR *pairs, *gain_pairs;
    long num_pairs, num_p = 0;
    long i, num_gains = 0;
    int status;

    if (UNDEF == te_init_meas_s_double_p_pair(epi, tm, eval))
        return (UNDEF);
    pairs = (DOUBLE_PARAM_PAIR *) tm->meas_params->param_values;
    num_pairs = tm->meas_params->num_params;
    for (i = 0; i < num_pairs; i++) {
        if (strcmp(pairs[i].name, "p"))
            continue;
        if (num_p) {
            fprintf(stderr,
                    "trec_eval: parameter 'p' of measure '%s' is given more than once\n",
                    tm->name);
            return (UNDEF);
        }
        if (pairs[i].value < 0.0 || pairs[i].value >= 1.0) {
            fprintf(stderr,
                    "trec_eval: persistence p=%g of measure '%s' is not in [0, 1)\n",
                    pairs[i].value, tm->name);
            return (UNDEF);
        }
        num_p = 1;
    }
    if (NULL == (gain_pairs = Malloc(num_pairs, DOUBLE_PARAM_PAIR)))
        return (UNDEF);
    for (i = 0; i < num_pairs; i++) {
        if (strcmp(pairs[i].name, "p"))
            gain_pairs[num_gains++] = pairs[i];
    }
    status = check_gain_params(tm, gain_pairs, num_gains);
    Free(gain_pairs);
    return (status);
}

int setup_gains(const TREC_MEAS * tm, const RES_RELS * res_rels, GAINS * gains)
{
    DOUBLE_PARAM_PAIR *pairs = NULL;
    long num_pairs = 0;
    long i, j;
    long num_gains;
    int error = 0;

    if (tm->meas_params) {
        pairs = (DOUBLE_PARAM_PAIR *) tm->meas_params->param_values;
        num_pairs = tm->meas_params->num_params;
    }

    if (NULL == (gains->rel_gains = Malloc(res_rels->num_rel_levels + num_pairs,
                                           REL_GAIN)))
        return (UNDEF);
    num_gains = 0;
    for (i = 0; i < num_pairs; i++) {
        if (!valid_int(pairs[i].name))
            continue;
        gains->rel_gains[num_gains].rel_level = str_to_long(pairs[i].name, &error);
        if (error) {
            fprintf(stderr, "trec_eval: Unusable relevance level '%s'\n",
                    pairs[i].name);
            return (UNDEF);
        }
        gains->rel_gains[num_gains].gain = (double) pairs[i].value;
        gains->rel_gains[num_gains].num_at_level = 0;
        num_gains++;
    }

    for (i = 0; i < res_rels->num_rel_levels; i++) {
        for (j = 0; j < num_gains && gains->rel_gains[j].rel_level != i; j++);
        if (j < num_gains)
            /* Was included in list of parameters. Update occurrence info */
            gains->rel_gains[j].num_at_level = res_rels->rel_levels[i];
        else {
            /* Not included in list of parameters. New gain level */
            gains->rel_gains[num_gains].rel_level = i;
            gains->rel_gains[num_gains].gain = (double) i;
            gains->rel_gains[num_gains].num_at_level = res_rels->rel_levels[i];
            num_gains++;
        }
    }

    /* Sort gains by increasing gain value */
    qsort((char *) gains->rel_gains,
          num_gains, sizeof(REL_GAIN), comp_rel_gain);

    gains->total_num_at_levels = 0;
    for (i = 0; i < num_gains; i++)
        gains->total_num_at_levels += gains->rel_gains[i].num_at_level;

    gains->num_gains = num_gains;
    return (1);
}

int comp_rel_gain(const void * ptr1, const void * ptr2)
{
    if (((REL_GAIN *) ptr1)->gain < ((REL_GAIN *) ptr2)->gain)
        return (-1);
    if (((REL_GAIN *) ptr1)->gain > ((REL_GAIN *) ptr2)->gain)
        return (1);
    return (0);
}

double get_gain(const long rel_level, const GAINS * gains)
{
    long i;
    for (i = 0; i < gains->num_gains; i++)
        if (rel_level == gains->rel_gains[i].rel_level)
            return (gains->rel_gains[i].gain);
    return (0.0);               /* Print Error ?? */
}
