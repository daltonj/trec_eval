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

int check_gain_params(const TREC_MEAS * tm, const char *named_param)
{
    DOUBLE_PARAM_PAIR *pairs =
        (DOUBLE_PARAM_PAIR *) tm->meas_params->param_values;
    long i, j, level_i = 0, level_j;
    int error = 0;
    int is_named;

    for (i = 0; i < tm->meas_params->num_params; i++) {
        is_named = named_param && 0 == strcmp(pairs[i].name, named_param);
        if (!is_named) {
            if (valid_int(pairs[i].name))
                level_i = str_to_long(pairs[i].name, &error);
            else
                error = 1;
            if (error) {
                fprintf(stderr, "trec_eval: Unusable relevance level '%s'\n",
                        pairs[i].name);
                return (UNDEF);
            }
        }
        for (j = 0; j < i; j++) {
            if (is_named) {
                if (0 == strcmp(pairs[i].name, pairs[j].name)) {
                    fprintf(stderr,
                            "trec_eval: parameter '%s' of measure '%s' is given more than once\n",
                            pairs[i].name, tm->name);
                    return (UNDEF);
                }
            } else if (!named_param || strcmp(pairs[j].name, named_param)) {
                level_j = str_to_long(pairs[j].name, &error);
                if (level_i == level_j) {
                    fprintf(stderr,
                            "trec_eval: relevance level %ld of measure '%s' is given more than once\n",
                            level_i, tm->name);
                    return (UNDEF);
                }
            }
        }
    }
    return (1);
}

int te_init_meas_s_double_p_gain(EPI * epi, TREC_MEAS * tm, TREC_EVAL * eval)
{
    if (UNDEF == te_init_meas_s_double_p_pair(epi, tm, eval))
        return (UNDEF);
    return check_gain_params(tm, NULL);
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
