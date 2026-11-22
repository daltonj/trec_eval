//! Measure values, computed by hand or printed by trec_eval for the same files.

mod common;

/// Cutoff lists and the measure values at each cutoff.
mod cutoffs {
    use super::common::{eval, rows, summary, write};

    /// Relevant documents at ranks 1 and 4 of 4.
    const TWO_RELEVANT_QRELS: &str = "q 0 d1 1\nq 0 d2 1\n";
    const TWO_RELEVANT_RUN: &str = "q Q0 d1 1 4 r\nq Q0 x1 2 3 r\nq Q0 x2 3 2 r\nq Q0 d2 4 1 r\n";

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
        let qrels = write("two-relevant.qrels", TWO_RELEVANT_QRELS);
        let run = write("two-relevant.run", TWO_RELEVANT_RUN);
        // (0.5 + 1 + 1) / 3, with the levels supplied in descending order.
        let out = eval(&["-m", "11pt_avg.1.0,0.5,0.5"], &qrels, &run);
        assert_eq!(summary(&out, "11pt_avg_1.0,0.5,0.5"), "0.8333");
    }
}

/// Cutoffs at the largest supported integer values.
mod parameters {
    use super::common::{eval, fixture, rows};

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn a_cutoff_of_i64_max_scores_as_trec_eval() {
        let (qrels, run) = (fixture("ab.qrels"), fixture("ab.run"));
        let max = i64::MAX;
        assert_eq!(
            rows(&eval(&["-m", &format!("P.{max}")], &qrels, &run)),
            [format!("P_{max} all 0.0000")]
        );
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn relstring_limit_larger_than_the_ranking_does_not_overflow() {
        let out = eval(
            &["-q", "-m", "relstring.18446744073709551615"],
            &fixture("ab.qrels"),
            &fixture("ab.run"),
        );
        assert_eq!(rows(&out), ["relstring_18446744073709551615 1 '10'"]);
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn utility_does_not_overflow_at_the_largest_collection_size() {
        // i64::MAX - 2 rounds to 2^63 as f64.
        let out = eval(
            &["-N", "9223372036854775807", "-m", "utility.0,0,0,1"],
            &fixture("ab.qrels"),
            &fixture("ab.run"),
        );
        assert_eq!(rows(&out), ["utility_0,0,0,1 all 9223372036854775808.0000"]);
    }
}

/// rbp normalizes the gains to [0, 1] by their range, which includes the gain 0 of an
/// unjudged document.
mod rbp {
    use super::common::{eval, fixture, summary, write};

    #[test]
    fn the_relevance_level_does_not_change_the_gains() {
        // Grade 1 gains 1 even when -l pads the count table through grade 3.
        let out = eval(
            &["-l", "3", "-m", "rbp"],
            &fixture("ab.qrels"),
            &fixture("ab.run"),
        );
        assert_eq!(summary(&out, "rbp"), "0.1000");
    }

    #[test]
    fn an_unjudged_document_gains_0_when_a_judged_gain_is_negative() {
        // Gains -1 and 1 normalize to 0 and 1. Unjudged x stays at 0; a contributes .1 * .9.
        let run = write("xa.run", "1 Q0 x 1 2 r\n1 Q0 a 2 1 r\n");
        let out = eval(&["-m", "rbp.0=-1,1=1"], &fixture("ab.qrels"), &run);
        assert_eq!(summary(&out, "rbp_0=-1,1=1"), "0.0900");
    }

    #[test]
    fn rbp_resid_accepts_a_gain_map_and_ignores_it() {
        // The residual is the weight of unjudged x, .1, and of the ranks after the last, .81.
        let run = write("xa.run", "1 Q0 x 1 2 r\n1 Q0 a 2 1 r\n");
        for (request, name) in [
            ("rbp_resid", "rbp_resid"),
            ("rbp_resid.1=0.5", "rbp_resid_1=0.5"),
        ] {
            let out = eval(&["-m", request], &fixture("ab.qrels"), &run);
            assert_eq!(summary(&out, name), "0.9100");
        }
    }

    #[test]
    fn the_normalization_range_includes_zero() {
        // Equal positive gains normalize to 1, giving .1 * (1 + .9).
        // Negative gains [-4, -2] normalize against zero to [0, .5], giving .1 * .5.
        for (gains, expected) in [("0=2,1=2", "0.1900"), ("0=-4,1=-2", "0.0500")] {
            let request = format!("rbp.{gains}");
            let out = eval(&["-m", &request], &fixture("ab.qrels"), &fixture("ab.run"));
            assert_eq!(summary(&out, &format!("rbp_{gains}")), expected);
        }
    }
}

/// The ndcg family and its two sums, dcg and ideal_dcg.
mod ndcg {
    use super::common::{eval, fixture, rows, summary, write};

    #[test]
    fn gain_maps_score_as_trec_eval() {
        // Gain 1 at rank 1 and gain 2 at rank 2: DCG = 1 + 2/log2(3).
        // The ideal order reverses them: 2 + 1/log2(3).
        let out = eval(
            &["-m", "ndcg.+1=1,00=2"],
            &fixture("ab.qrels"),
            &fixture("ab.run"),
        );
        assert_eq!(rows(&out), ["ndcg_+1=1,00=2 all 0.8597"]);
    }

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
        // x and a gain 1: DCG is 1, then 1.6309. With R = 1 and two gain-bearing results,
        // the sum is 1 + 1.6309 + (1 - 2) * 1.6309 = 1.
        let run = write("xa.run", "1 Q0 x 1 2 r\n1 Q0 a 2 1 r\n");
        let out = eval(&["-m", "ndcg_rel.-1=1"], &fixture("ab.qrels"), &run);
        assert_eq!(summary(&out, "ndcg_rel_-1=1"), "1.0000");
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

    #[test]
    fn dcg_sums_use_independent_gain_maps() {
        let qrels = write("dcg.qrels", "1 0 a 1\n1 0 b 2\n1 0 c 0\n");
        let run = write("dcg.run", "1 Q0 a 1 3 r\n1 Q0 c 2 2 r\n1 Q0 b 3 1 r\n");
        // dcg = 1 + 2/log2(4) = 2; ideal = 2 + 1/log2(3) = 2.6309.
        // A local gain -1 for c adds -1/log2(3) to dcg, but not to the ideal.
        for (options, expected) in [
            (
                &["-m", "dcg.0=-1", "-m", "ideal_dcg.0=-1", "-m", "ndcg"][..],
                &[
                    "dcg_0=-1 all 1.3691",
                    "ideal_dcg_0=-1 all 2.6309",
                    "ndcg all 0.7602",
                ][..],
            ),
            // Global a -> 3 gives ideal 3 + 2/log2(3). Local b -> 5 overrides it:
            // dcg = 1 + 5/2 = 3.5.
            (
                &["--global-gains", "1=3", "-m", "dcg.2=5", "-m", "ideal_dcg"],
                &["dcg_2=5 all 3.5000", "ideal_dcg all 4.2619"],
            ),
        ] {
            assert_eq!(rows(&eval(options, &qrels, &run)), expected);
        }
        // An empty ranking still has an ideal sum. The second qrels topic shares no run topic.
        let run = write("other.run", "2 Q0 x 1 1 r\n");
        assert_eq!(
            rows(&eval(&["-c", "-m", "dcg", "-m", "ideal_dcg"], &qrels, &run)),
            ["dcg all 0.0000", "ideal_dcg all 2.6309"]
        );
    }
}

/// gm_map and gm_bpref, which floor a topic's value at 1e-5 before taking its log.
mod geometric_means {
    use super::common::{eval, rows, write};

    #[test]
    fn a_missing_topic_counts_once_under_complete() {
        let qrels = write("two.qrels", "1 0 a 1\n2 0 b 1\n");
        let run = write("one.run", "1 Q0 a 1 1 r\n");
        // exp((ln 1 + ln 1e-5) / 2) = sqrt(1e-5).
        let out = eval(&["-c", "-m", "gm_map", "-m", "gm_bpref"], &qrels, &run);
        assert_eq!(rows(&out), ["gm_map all 0.0032", "gm_bpref all 0.0032"]);
    }
}
