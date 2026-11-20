//! Measure values, computed by hand or printed by trec_eval for the same files.

mod common;

/// Measures with a list of cutoffs, which te-rust sorts as trec_eval does.
mod cutoffs {
    use super::common::{eval, fixture, rows, summary, write};

    /// Relevant documents a, c, e, f and h, of which the run ranks a, c, e and f at ranks 1, 4,
    /// 7 and 8, with unjudged documents at ranks 2 and 5.
    const MIXED_QRELS: &str =
        "q 0 a 1\nq 0 b 0\nq 0 c 1\nq 0 d 0\nq 0 e 1\nq 0 f 1\nq 0 g 0\nq 0 h 1\n";
    const MIXED_RUN: &str = "q Q0 a 1 19 r\nq Q0 z 2 18 r\nq Q0 b 3 17 r\nq Q0 c 4 16 r\n\
                             q Q0 y 5 15 r\nq Q0 d 6 14 r\nq Q0 e 7 13 r\nq Q0 f 8 12 r\n";

    /// Relevant documents at ranks 1 and 4 of 4.
    const TWO_RELEVANT_QRELS: &str = "q 0 d1 1\nq 0 d2 1\n";
    const TWO_RELEVANT_RUN: &str = "q Q0 d1 1 4 r\nq Q0 x1 2 3 r\nq Q0 x2 3 2 r\nq Q0 d2 4 1 r\n";

    /// The value at a cutoff cannot depend on which other cutoffs are listed, or in what order.
    fn assert_order_free(options: &[&str], measure: &str, low: &str, high: &str) {
        let (qrels, run) = (
            write("mixed.qrels", MIXED_QRELS),
            write("mixed.run", MIXED_RUN),
        );
        let printed = |params: &str| {
            let request = format!("{measure}.{params}");
            let mut args = options.to_vec();
            args.extend(["-m", &request]);
            rows(&eval(&args, &qrels, &run))
        };
        let mut alone = printed(low);
        alone.extend(printed(high));
        assert_eq!(
            printed(&format!("{low},{high}")),
            alone,
            "{measure} ascending"
        );
        assert_eq!(
            printed(&format!("{high},{low}")),
            alone,
            "{measure} descending"
        );
    }

    #[test]
    fn every_cutoff_family_is_order_free() {
        for measure in [
            "P",
            "recall",
            "success",
            "map_cut",
            "ndcg_cut",
            "relative_P",
            "unj",
        ] {
            assert_order_free(&[], measure, "2", "5");
        }
        assert_order_free(&[], "iprec_at_recall", "0.2", "0.8");
        assert_order_free(&[], "Rprec_mult", "0.5", "2.0");
        assert_order_free(&["-R", "qrels_jg"], "P_avgjg", "2", "5");
        assert_order_free(&["-R", "qrels_jg"], "Rprec_mult_avgjg", "0.5", "2.0");
    }

    #[test]
    fn rprec_mult_reaches_every_multiple_when_descending() {
        // R = 5. Rprec_mult_2.00 is precision at rank 10: a, c, e and f are relevant, 4 / 10.
        let (qrels, run) = (
            write("mixed.qrels", MIXED_QRELS),
            write("mixed.run", MIXED_RUN),
        );
        let out = eval(&["-m", "Rprec_mult.2.0,0.5"], &qrels, &run);
        assert_eq!(summary(&out, "Rprec_mult_2.00"), "0.4000");
    }

    #[test]
    fn rows_print_in_ascending_cutoff_order_like_trec_eval() {
        let (qrels, run) = (
            write("mixed.qrels", MIXED_QRELS),
            write("mixed.run", MIXED_RUN),
        );
        let rows = rows(&eval(&["-m", "P.5,2"], &qrels, &run));
        let names: Vec<_> = rows
            .iter()
            .map(|row| row.split(' ').next().unwrap())
            .collect();
        assert_eq!(names, ["P_2", "P_5"]);
    }

    #[test]
    fn descending_lists_match_hand_calculations() {
        let (qrels, run) = (fixture("six-relevant.qrels"), fixture("six-relevant.run"));
        // All six relevant documents are in the top six: map_cut_2 = (1 + 1) / 6 and
        // map_cut_5 = (1 + 1 + 1 + 1 + 1) / 6.
        let out = eval(&["-m", "map_cut.5,2"], &qrels, &run);
        assert_eq!(rows(&out), ["map_cut_2 all 0.3333", "map_cut_5 all 0.8333"]);
        // relative_P cannot exceed 1.
        let out = eval(&["-m", "relative_P.5,2"], &qrels, &run);
        assert_eq!(
            rows(&out),
            ["relative_P_2 all 1.0000", "relative_P_5 all 1.0000"]
        );
    }

    #[test]
    fn utility_coefficients_keep_their_positions() {
        // rel_ret * 2 - nonrel_ret * 1 = 6 * 2. Sorting the list would give -18.
        let (qrels, run) = (fixture("six-relevant.qrels"), fixture("six-relevant.run"));
        let out = eval(&["-m", "utility.2,-1,0,0"], &qrels, &run);
        assert_eq!(summary(&out, "utility_2,-1,0,0"), "12.0000");
    }

    #[test]
    fn eleven_pt_avg_does_not_depend_on_list_order() {
        // Interpolated precision is 1 at recall 0.5 and 2/4 at recall 1.0, so the average is
        // 0.75 in either order. trec_eval reads 11pt_avg with get_double_params, which does not
        // sort, and walks the list from its last entry, so it prints 1.0000 for the descending
        // list. The row name keeps the levels as given, as in trec_eval.
        let qrels = write("two-relevant.qrels", TWO_RELEVANT_QRELS);
        let run = write("two-relevant.run", TWO_RELEVANT_RUN);
        assert_eq!(
            rows(&eval(&["-m", "11pt_avg.0.5,1.0"], &qrels, &run)),
            ["11pt_avg_0.5,1.0 all 0.7500"]
        );
        assert_eq!(
            rows(&eval(&["-m", "11pt_avg.1.0,0.5"], &qrels, &run)),
            ["11pt_avg_1.0,0.5 all 0.7500"]
        );
    }

    #[test]
    fn eleven_pt_avg_counts_a_repeated_level_each_time() {
        // (1 + 1 + 0.5) / 3, whatever the order.
        let qrels = write("two-relevant.qrels", TWO_RELEVANT_QRELS);
        let run = write("two-relevant.run", TWO_RELEVANT_RUN);
        for levels in ["0.5,0.5,1.0", "0.5,1.0,0.5", "1.0,0.5,0.5"] {
            let out = eval(&["-m", &format!("11pt_avg.{levels}")], &qrels, &run);
            assert_eq!(
                summary(&out, &format!("11pt_avg_{levels}")),
                "0.8333",
                "{levels}"
            );
        }
    }
}

/// rbp normalises the gains to [0, 1] by their range, which includes the gain 0 of an
/// unjudged document.
mod rbp {
    use super::common::{eval, fixture, rows, summary, write};

    #[test]
    fn gain_maps_and_persistence() {
        let (qrels, run) = (fixture("graded.qrels"), fixture("graded.run"));
        for (options, row) in [
            // The gains normalise by the top grade 3: d = 1, a = 2/3, c = 1/3, at ranks 1, 3
            // and 5. 0.1 * (1 + 2/3 * 0.81 + 1/3 * 0.6561) = 0.1759
            (&["-m", "rbp"][..], "rbp all 0.1759"),
            // Gains 1 -> 3, 2 -> 9, 3 -> 3 normalise by 9: d = 1/3, a = 1, c = 1/3.
            // 0.1 * (1/3 + 1 * 0.81 + 1/3 * 0.6561) = 0.1362
            (&["-m", "rbp.1=3,2=9"], "rbp_1=3,2=9 all 0.1362"),
            // 0.5 * (1/3 + 1 * 0.25 + 1/3 * 0.0625) = 0.3021
            (&["-m", "rbp.p=0.5,1=3,2=9"], "rbp_p=0.5,1=3,2=9 all 0.3021"),
            // 0.5 * (1 + 2/3 * 0.25 + 1/3 * 0.0625) = 0.5938
            (&["-m", "rbp.p=0.5"], "rbp_p=0.5 all 0.5938"),
            (
                &["--global-gains", "1=3,2=9", "-m", "rbp"],
                "rbp all 0.1362",
            ),
            // The measure's gain map takes precedence, as for ndcg.
            (
                &["--global-gains", "2=1,1=9", "-m", "rbp.1=3,2=9"],
                "rbp_1=3,2=9 all 0.1362",
            ),
        ] {
            assert_eq!(rows(&eval(options, &qrels, &run)), [row], "{options:?}");
        }
    }

    #[test]
    fn the_relevance_level_does_not_change_the_gains() {
        // a is relevant at rank 1, so rbp = (1 - 0.9) * 1.
        for level in ["1", "2", "3", "4", "100"] {
            let out = eval(
                &["-l", level, "-m", "rbp"],
                &fixture("ab.qrels"),
                &fixture("ab.run"),
            );
            assert_eq!(summary(&out, "rbp"), "0.1000", "-l {level}");
        }
        for level in ["1", "2", "3", "4", "5", "100"] {
            let out = eval(
                &["-l", level, "-m", "rbp"],
                &fixture("graded.qrels"),
                &fixture("graded.run"),
            );
            assert_eq!(summary(&out, "rbp"), "0.1759", "-l {level}");
        }
    }

    #[test]
    fn an_unjudged_document_gains_0_when_a_judged_gain_is_negative() {
        // Gains 0 -> -1, 1 -> 1 normalise to 0 and 1. x at rank 1 is unjudged and gains 0, and
        // a at rank 2 gives 0.1 * 0.9 = 0.0900.
        let run = write("xab.run", "1 Q0 x 1 3 r\n1 Q0 a 2 2 r\n1 Q0 b 3 1 r\n");
        let out = eval(
            &["--global-gains", "0=-1,1=1", "-m", "rbp"],
            &fixture("ab.qrels"),
            &run,
        );
        assert_eq!(summary(&out, "rbp"), "0.0900");
        // Gains -2, 0, 6 for c, b, a normalise to 0, 0.25, 1. u at rank 2 is unjudged and gains
        // 0. 0.1 * (0 + 0 + 1 * 0.81 + 0.25 * 0.729) = 0.099225
        let qrels = write("three-grades.qrels", "t 0 a 2\nt 0 b 1\nt 0 c 0\n");
        let run = write(
            "three-grades.run",
            "t Q0 c 1 4 r\nt Q0 u 2 3 r\nt Q0 a 3 2 r\nt Q0 b 4 1 r\n",
        );
        let out = eval(
            &["--global-gains", "0=-2,1=0,2=6", "-m", "rbp"],
            &qrels,
            &run,
        );
        assert_eq!(summary(&out, "rbp"), "0.0992");
    }

    #[test]
    fn the_normalisation_range_includes_zero_for_both_gain_sources() {
        // `one` judges a alone and ranks it first; `both` also judges b, at grade 0 and rank 2.
        let one = (
            write("one.qrels", "1 0 a 1\n"),
            write("one.run", "1 Q0 a 1 3 r\n"),
        );
        let both = (fixture("ab.qrels"), fixture("ab.run"));
        for (gains, in_one, in_both) in [
            ("0=2,1=2", "0.1000", "0.1900"),
            ("0=-4,1=-2", "0.0500", "0.0500"),
            ("0=1,1=2", "0.1000", "0.1450"),
            ("0=-2,1=-1", "0.0500", "0.0500"),
        ] {
            let (measure, row) = (format!("rbp.{gains}"), format!("rbp_{gains}"));
            for (options, row) in [
                (&["-m", &measure][..], row.as_str()),
                (&["--global-gains", gains, "-m", "rbp"], "rbp"),
            ] {
                for ((qrels, run), value) in [(&one, in_one), (&both, in_both)] {
                    assert_eq!(
                        rows(&eval(options, qrels, run)),
                        [format!("{row} all {value}")],
                        "{qrels:?}"
                    );
                }
            }
        }
        // An absent level keeps gain 0 when every gain in the table is negative.
        let run = write("xa.run", "1 Q0 x 1 3 r\n1 Q0 a 2 2 r\n");
        let out = eval(&["-m", "rbp.0=-4,1=-2"], &one.0, &run);
        assert_eq!(summary(&out, "rbp_0=-4,1=-2"), "0.0450");
    }
}

/// The ndcg family and its two sums, dcg and ideal_dcg.
mod ndcg {
    use super::common::{assert_refused, eval, fixture, rows, summary, write};
    use std::process::Output;

    #[test]
    fn ndcg_rel_sums_its_terms_with_either_sign() {
        // b has grade 0 and rank 1, a grade 1 and rank 2. By default a alone gains: its term is
        // (1 / log2(3)) / 1 = 0.6309.
        let run = write("ba.run", "1 Q0 b 1 2 r\n1 Q0 a 2 1 r\n");
        let out = eval(&["-m", "ndcg_rel"], &fixture("ab.qrels"), &run);
        assert_eq!(summary(&out, "ndcg_rel"), "0.6309");
        // With gain -1 for b, DCG at a is -1 + 1 / log2(3) < 0, which reports 0.
        let out = eval(&["-m", "ndcg_rel.0=-1,1=1"], &fixture("ab.qrels"), &run);
        assert_eq!(summary(&out, "ndcg_rel_0=-1,1=1"), "0.0000");
    }

    #[test]
    fn ndcg_rel_final_term_subtracts_when_num_rel_ret_exceeds_num_rel() {
        // Gain 1 on x, a and y, with num_rel 1, num_rel_ret 3 and ideal DCG 1. DCG is 1, 1.6309
        // and 2.1309 at ranks 1 to 3, so the sum is 1 + 1.6309 + 2.1309 + (1 - 3) * 2.1309 = 0.5,
        // over one relevant document.
        let run = write("xay.run", "1 Q0 x 1 3 r\n1 Q0 a 2 2 r\n1 Q0 y 3 1 r\n");
        let out = eval(&["-m", "ndcg_rel.-1=1"], &fixture("ab.qrels"), &run);
        assert_eq!(summary(&out, "ndcg_rel_-1=1"), "0.5000");
    }

    /// The gains and the discount of ndcg_p, which docs/measures/gains.md describes, on topic 1
    /// of topics.qrels: d1 has grade 1, d2 grade 0 and d3 grade 2, ranked in that order.
    #[test]
    fn ndcg_p_uses_level_gains_and_the_original_discount() {
        let run = write(
            "topic-1.run",
            "1 Q0 d1 1 3 r\n1 Q0 d2 2 2 r\n1 Q0 d3 3 1 r\n",
        );
        let printed = |measure: &str| rows(&eval(&["-m", measure], &fixture("topics.qrels"), &run));
        // Rank 1 is not discounted and rank i >= 2 is divided by log2(i):
        // DCG = 1 + 0/1 + 2/log2(3) = 2.26186, and the ideal 2, 1, 0 gives 2 + 1/1 = 3.
        assert_eq!(printed("ndcg_p"), ["ndcg_p all 0.7540"]);
        // Exponential gains 2^r - 1 would give (1 + 3/log2(3)) / (3 + 1) = 0.7232.
        // ndcg divides rank i by log2(i + 1): (1 + 2/2) / (2 + 1/log2(3)) = 0.7602.
        assert_eq!(printed("ndcg"), ["ndcg all 0.7602"]);
        // (3.5 + 9/log2(3)) / (9 + 3.5/1) = 9.17837 / 12.5.
        assert_eq!(
            printed("ndcg_p.1=3.5,2=9.0"),
            ["ndcg_p_1=3.5,2=9.0 all 0.7343"]
        );
        // Level 0 may have a gain, and a gain may be negative. The ideal order is d3 (2), d2
        // (1), and stops at d1 (-1): (-1 + 1/1 + 2/log2(3)) / (2 + 1/1).
        assert_eq!(printed("ndcg_p.0=1,1=-1"), ["ndcg_p_0=1,1=-1 all 0.4206"]);
    }

    /// Topic 1 judges d1 at grade 1, d2 at 2 and d3 at 0, and the run ranks d1, x (unjudged),
    /// d2, d3. Topic 2 judges d1 at 0 and d2 at 1, and the run ranks d1, d2. trec_eval 10.0 prints
    /// the same value for each measure requested alone.
    const DCG_QRELS: &str = "1 0 d1 1\n1 0 d2 2\n1 0 d3 0\n2 0 d1 0\n2 0 d2 1\n";
    const DCG_RUN: &str = "1 Q0 d1 1 4 r\n1 Q0 x 2 3 r\n1 Q0 d2 3 2 r\n1 Q0 d3 4 1 r\n\
                           2 Q0 d1 1 2 r\n2 Q0 d2 2 1 r\n";

    fn dcg_eval(options: &[&str]) -> Output {
        eval(
            options,
            &write("dcg.qrels", DCG_QRELS),
            &write("dcg.run", DCG_RUN),
        )
    }

    #[test]
    fn dcg_and_ideal_dcg_print_the_two_sums_of_ndcg() {
        // Topic 1: dcg = 1/log2(2) + 2/log2(4) = 2; the ideal order 2, 1 gives
        // ideal_dcg = 2 + 1/log2(3) = 2.6309, and ndcg = 2 / 2.6309 = 0.7602.
        // Topic 2: dcg = 1/log2(3) = 0.6309, ideal_dcg = 1, ndcg = 0.6309.
        // Each summary is the mean of its own topic values: (2 + 0.6309) / 2 = 1.3155,
        // (2.6309 + 1) / 2 = 1.8155 and (0.7602 + 0.6309) / 2 = 0.6956. The mean of ndcg is
        // not the ratio of the means.
        assert_eq!(
            rows(&dcg_eval(&[
                "-q",
                "-m",
                "dcg",
                "-m",
                "ideal_dcg",
                "-m",
                "ndcg"
            ])),
            [
                "dcg 1 2.0000",
                "ideal_dcg 1 2.6309",
                "ndcg 1 0.7602",
                "dcg 2 0.6309",
                "ideal_dcg 2 1.0000",
                "ndcg 2 0.6309",
                "dcg all 1.3155",
                "ideal_dcg all 1.8155",
                "ndcg all 0.6956",
            ]
        );
    }

    #[test]
    fn gain_map_sets_the_gains_and_the_row_name() {
        // Gain 3 for grade 1. Topic 1: dcg = 3 + 2/log2(4) = 4, ideal_dcg = 3 + 2/log2(3) =
        // 4.2619. Topic 2: dcg = 3/log2(3) = 1.8928, ideal_dcg = 3.
        assert_eq!(
            rows(&dcg_eval(&["-q", "-m", "dcg.1=3"])),
            ["dcg_1=3 1 4.0000", "dcg_1=3 2 1.8928", "dcg_1=3 all 2.9464"]
        );
        assert_eq!(
            rows(&dcg_eval(&["-q", "-m", "ideal_dcg.1=3"])),
            [
                "ideal_dcg_1=3 1 4.2619",
                "ideal_dcg_1=3 2 3.0000",
                "ideal_dcg_1=3 all 3.6309"
            ]
        );
    }

    #[test]
    fn gains_that_are_not_positive_count_in_dcg_only() {
        // Gain -1 for grade 0. dcg adds it at every rank: topic 1 gains -1/log2(5) at d3, so
        // 2 - 0.4307 = 1.5693, and topic 2 gains -1 at d1, so -1 + 0.6309 = -0.3691.
        // ideal_dcg stops at the first gain that is not positive and is unchanged.
        assert_eq!(
            rows(&dcg_eval(&["-q", "-m", "dcg.0=-1", "-m", "ideal_dcg.0=-1"])),
            [
                "dcg_0=-1 1 1.5693",
                "ideal_dcg_0=-1 1 2.6309",
                "dcg_0=-1 2 -0.3691",
                "ideal_dcg_0=-1 2 1.0000",
                "dcg_0=-1 all 0.6001",
                "ideal_dcg_0=-1 all 1.8155",
            ]
        );
        // With gain 0 for grade 1 as well, topic 2 has no positive gain: ideal_dcg is 0, and
        // dcg still sums the ranking, -1 at d1.
        assert_eq!(
            rows(&dcg_eval(&[
                "-q",
                "-m",
                "dcg.0=-1,1=0",
                "-m",
                "ideal_dcg.0=-1,1=0"
            ])),
            [
                "dcg_0=-1,1=0 1 0.5693",
                "ideal_dcg_0=-1,1=0 1 2.0000",
                "dcg_0=-1,1=0 2 -1.0000",
                "ideal_dcg_0=-1,1=0 2 0.0000",
                "dcg_0=-1,1=0 all -0.2153",
                "ideal_dcg_0=-1,1=0 all 1.0000",
            ]
        );
    }

    #[test]
    fn each_measure_reads_only_its_own_gain_map() {
        // Each row is the value that the measure prints alone. trec_eval 10.0 shares one
        // parameter list among dcg, ideal_dcg and ndcg, and crashes on these requests.
        assert_eq!(
            rows(&dcg_eval(&[
                "-m",
                "dcg.1=3",
                "-m",
                "ideal_dcg",
                "-m",
                "ndcg"
            ])),
            [
                "dcg_1=3 all 2.9464",
                "ideal_dcg all 1.8155",
                "ndcg all 0.6956"
            ]
        );
        // ndcg.1=3: topic 1 is 4 / 4.2619 = 0.9386 and topic 2 is 1.8928 / 3 = 0.6309.
        assert_eq!(
            rows(&dcg_eval(&["-m", "dcg", "-m", "ndcg.1=3"])),
            ["dcg all 1.3155", "ndcg_1=3 all 0.7847"]
        );
    }

    #[test]
    fn global_gains_apply_unless_the_measure_has_a_gain_map() {
        assert_eq!(
            rows(&dcg_eval(&[
                "--global-gains",
                "1=3",
                "-m",
                "dcg",
                "-m",
                "ideal_dcg"
            ])),
            ["dcg all 2.9464", "ideal_dcg all 3.6309"]
        );
        // dcg.2=5 keeps gain 1 for grade 1: topic 1 is 1 + 5/log2(4) = 3.5 and topic 2 is
        // 0.6309, as `-m dcg.2=5` alone.
        assert_eq!(
            rows(&dcg_eval(&["--global-gains", "1=3", "-m", "dcg.2=5"])),
            ["dcg_2=5 all 2.0655"]
        );
    }

    #[test]
    fn complete_set_scores_a_missing_topic_as_an_empty_ranking() {
        // Topic 3 judges d1 at grade 2, and the run lacks it. Under -c it has dcg 0 and
        // ideal_dcg 2, the gain of its one judged document at rank 1. The means are over 3
        // topics: 2.6309 / 3 = 0.8770 and 5.6309 / 3 = 1.8770.
        let qrels = write("dcg.qrels", format!("{DCG_QRELS}3 0 d1 2\n"));
        let out = eval(
            &["-c", "-q", "-m", "dcg", "-m", "ideal_dcg"],
            &qrels,
            &write("dcg.run", DCG_RUN),
        );
        assert_eq!(
            rows(&out),
            [
                "dcg 1 2.0000",
                "ideal_dcg 1 2.6309",
                "dcg 2 0.6309",
                "ideal_dcg 2 1.0000",
                "dcg 3 0.0000",
                "ideal_dcg 3 2.0000",
                "dcg all 0.8770",
                "ideal_dcg all 1.8770",
            ]
        );
    }

    #[test]
    fn unusable_gain_maps_are_refused_as_for_ndcg() {
        for (measure, message) in [
            ("dcg.1=x", "invalid gain 'x' for relevance level 1"),
            ("ideal_dcg.x=1", "invalid relevance level 'x'"),
            ("dcg.1=2,1=5", "relevance level 1 is given more than once"),
            ("ideal_dcg.5,5", "expected key=value, got '5'"),
        ] {
            assert_refused(&dcg_eval(&["-m", measure]), message);
        }
    }
}

/// gm_map and gm_bpref, which floor a topic's value at 1e-5 before taking its log.
mod geometric_means {
    use super::common::{eval, summary, test_data, write};
    use std::path::PathBuf;

    /// AP and bpref are 1 on topic 1 and 0.5 on topic 2, and topic 3 has nothing relevant
    /// retrieved: exp((ln 1 + ln 0.5 + ln 1e-5) / 3) = 0.0171.
    fn three_topics() -> PathBuf {
        write(
            "three-topics.qrels",
            "1 0 a 1\n2 0 c 1\n2 0 f 1\n2 0 d 0\n3 0 e 1\n",
        )
    }

    fn run_without_topic_3() -> PathBuf {
        write(
            "no-topic-3.run",
            "1 Q0 a 1 1.0 r\n2 Q0 c 1 2.0 r\n2 Q0 d 2 1.0 r\n",
        )
    }

    /// Topic 3 retrieves only an unjudged document.
    fn run_with_topic_3() -> PathBuf {
        write(
            "topic-3.run",
            "1 Q0 a 1 1.0 r\n2 Q0 c 1 2.0 r\n2 Q0 d 2 1.0 r\n3 Q0 x 1 1.0 r\n",
        )
    }

    #[test]
    fn a_missing_topic_counts_once_under_complete() {
        let out = eval(
            &["-c", "-m", "gm_map", "-m", "gm_bpref"],
            &three_topics(),
            &run_without_topic_3(),
        );
        assert_eq!(summary(&out, "gm_map"), "0.0171");
        assert_eq!(summary(&out, "gm_bpref"), "0.0171");
    }

    #[test]
    fn a_missing_topic_scores_like_a_topic_with_nothing_relevant() {
        let missing = eval(
            &["-c", "-m", "gm_map"],
            &three_topics(),
            &run_without_topic_3(),
        );
        let unjudged = eval(
            &["-c", "-m", "gm_map"],
            &three_topics(),
            &run_with_topic_3(),
        );
        assert_eq!(summary(&missing, "gm_map"), summary(&unjudged, "gm_map"));
        let out = eval(&["-m", "gm_map"], &three_topics(), &run_with_topic_3());
        assert_eq!(summary(&out, "gm_map"), "0.0171");
    }

    #[test]
    fn complete_values_match_trec_eval_reference_output() {
        // out.test.aqc is trec_eval's expected output for `-q -c -m all_trec`.
        let options = ["-q", "-c", "-m", "all_trec"];
        let out = eval(
            &options,
            &test_data("qrels.test"),
            &test_data("results.trunc"),
        );
        let reference = std::fs::read_to_string(test_data("out.test.aqc")).unwrap();
        for measure in ["gm_map", "gm_bpref"] {
            let expected = reference
                .lines()
                .map(|line| line.split_whitespace().collect::<Vec<_>>())
                .find(|fields| fields[0] == measure && fields[1] == "all")
                .unwrap()[2]
                .to_string();
            assert_eq!(summary(&out, measure), expected, "{measure}");
        }
    }
}

/// Relevance levels of 0 and below, at which nonrelevant and unjudged documents can count as
/// relevant, as in trec_eval.
mod negative_levels {
    use super::common::{eval, fixture, rows, summary, test_data, write};
    use std::path::{Path, PathBuf};

    /// The trec_eval binary in `$TREC_EVAL_DIR`, which `regression.rs` reads from the
    /// workspace root, when that variable is set.
    fn reference_trec_eval() -> Option<PathBuf> {
        let dir = std::env::var_os("TREC_EVAL_DIR")?;
        Some(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join(dir)
                .join("trec_eval"),
        )
    }

    #[test]
    fn eleven_pt_avg_matches_the_c_reverse_scan_for_unjudged_documents() {
        // a is relevant, b nonrelevant and u has grade -1; x is not judged. Each run ranks the
        // documents its name spells.
        let qrels = write("negative.qrels", "q 0 a 1\nq 0 b 0\nq 0 u -1\n");
        // Columns are -l-2, -l-1, -l0 and -l1, as printed by trec_eval.
        for (docs, values) in [
            ("xa", ["0.3636", "0.3636", "0.3636", "0.5000"]),
            ("ax", ["0.3636", "0.3636", "0.7273", "1.0000"]),
            ("ua", ["0.3636", "0.3636", "0.3636", "0.5000"]),
            ("au", ["0.3636", "0.7273", "0.7273", "1.0000"]),
            ("xua", ["0.2424", "0.2424", "0.2424", "0.3333"]),
            ("aux", ["0.2424", "0.2424", "0.7273", "1.0000"]),
            ("xu", ["0.0000", "0.0000", "0.0000", "0.0000"]),
            ("ab", ["1.0000", "1.0000", "1.0000", "1.0000"]),
        ] {
            let run: String = docs
                .chars()
                .enumerate()
                .map(|(i, docno)| format!("q Q0 {docno} {} {} run\n", i + 1, 9 - i))
                .collect();
            let run = write(&format!("{docs}.run"), run);
            for (level, value) in [-2, -1, 0, 1].into_iter().zip(values) {
                let relevant = match (docs, level) {
                    ("ab", ..=0) => "2",
                    ("xu", _) => "0",
                    _ => "1",
                };
                let level = format!("-l{level}");
                let options = [level.as_str(), "-m", "11pt_avg", "-m", "num_rel_ret"];
                let out = eval(&options, &qrels, &run);
                let printed = (summary(&out, "num_rel_ret"), summary(&out, "11pt_avg"));
                assert_eq!(
                    printed,
                    (relevant.to_string(), value.to_string()),
                    "{docs}, {level}"
                );
                if let Some(trec_eval) = reference_trec_eval() {
                    let reference = std::process::Command::new(trec_eval)
                        .args(options)
                        .arg(&qrels)
                        .arg(&run)
                        .output()
                        .unwrap();
                    let expected = (
                        summary(&reference, "num_rel_ret"),
                        summary(&reference, "11pt_avg"),
                    );
                    assert_eq!(printed, expected, "trec_eval: {docs}, {level}");
                }
            }
        }
    }

    #[test]
    fn rprec_mult_avgjg_stops_its_reverse_scan_at_zero_relevant_documents() {
        let qrels = write("one.qrels", "1 g a 1\n");
        let run = write("axy.run", "1 Q0 a 1 3 r\n1 Q0 x 2 2 r\n1 Q0 y 3 1 r\n");
        for level in ["-l-1", "-l-2"] {
            let out = eval(
                &["-R", "qrels_jg", level, "-m", "Rprec_mult_avgjg.0.5,1,2,4"],
                &qrels,
                &run,
            );
            assert_eq!(
                rows(&out),
                [
                    "Rprec_mult_avgjg_0.50 all 0.0000",
                    "Rprec_mult_avgjg_1.00 all 0.0000",
                    "Rprec_mult_avgjg_2.00 all 0.0000",
                    "Rprec_mult_avgjg_4.00 all 0.2500",
                ],
                "{level}"
            );
        }
    }

    #[test]
    fn utility_counts_only_judged_documents_as_relevant_retrieved() {
        // ab.qrels judges a 1 and b 0, and the run ranks a, x, y. At level -1, a and b are
        // relevant, and x and y are not in the qrels. Retrieved relevant 1 (a), retrieved
        // nonrelevant 2, missed relevant 1 (b), so 1 - 2 + 1 = 0, and with 10 documents
        // d = 10 + 1 - 3 - 2 = 6, as trec_eval -l-1 prints. At level 0, a and b are relevant:
        // 1 - 2 + 1 = 0. At level 1, only a is: 1 - 2 + 0 = -1. trec_eval prints the same.
        let run = write("axy.run", "1 Q0 a 1 3 r\n1 Q0 x 2 2 r\n1 Q0 y 3 1 r\n");
        for (options, row, value) in [
            (
                &["-l-1", "-m", "utility.1,-1,1,0"][..],
                "utility_1,-1,1,0",
                "0.0000",
            ),
            (
                &["-l-1", "-N", "10", "-m", "utility.0,0,0,1"],
                "utility_0,0,0,1",
                "6.0000",
            ),
            (
                &["-l0", "-m", "utility.1,-1,1,0"],
                "utility_1,-1,1,0",
                "0.0000",
            ),
            (
                &["-l1", "-m", "utility.1,-1,1,0"],
                "utility_1,-1,1,0",
                "-1.0000",
            ),
        ] {
            assert_eq!(
                summary(&eval(options, &fixture("ab.qrels"), &run), row),
                value,
                "{options:?}"
            );
        }
    }

    #[test]
    fn utility_matches_trec_eval_at_a_negative_level_on_its_test_files() {
        let (qrels, run) = (test_data("qrels.test"), test_data("results.test"));
        assert_eq!(
            summary(&eval(&["-l-1", "-m", "utility"], &qrels, &run), "utility"),
            "-8.0000"
        );
        let options = ["-l-1", "-N", "100000", "-m", "utility.2,-1,1,0.5"];
        assert_eq!(
            summary(&eval(&options, &qrels, &run), "utility_2,-1,1,0.5"),
            "50478.5000"
        );
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn utility_does_not_overflow_at_the_largest_collection_size() {
        let run = write("axy.run", "1 Q0 a 1 3 r\n1 Q0 x 2 2 r\n1 Q0 y 3 1 r\n");
        let options = ["-N", "9223372036854775807", "-m", "utility.0,0,0,1"];
        let out = eval(&options, &fixture("ab.qrels"), &run);
        assert_eq!(summary(&out, "utility_0,0,0,1"), "9223372036854775808.0000");
    }
}
