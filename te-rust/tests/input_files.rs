//! What te-rust makes of the qrels and run files it reads. The parsers' own cases are unit
//! tests in `src/io/parsers.rs`.

mod common;

use common::{assert_refused, eval, rows, stderr, summary, test_data, write};

/// Scores that trec_eval reads with `strtod`. The qrels grade a, b and c 1, 2 and 3, so the
/// relstring of a ranking spells it: '213' ranks b first. trec_eval prints the same relstrings
/// and warnings.
#[test]
fn scores_are_read_as_strtod_reads_them() {
    let qrels = write("abc.qrels", "1 0 a 1\n1 0 b 2\n1 0 c 3\n");
    let cases: &[([&str; 3], &str, &[&str])] = &[
        (["5", "inf", "3"], "'213'", &[]),
        (["5", "infinity", "3"], "'213'", &[]),
        (["5", "-inf", "3"], "'132'", &[]),
        // Tied scores rank the docno that sorts last first, infinite ones too.
        (["inf", "inf", "-inf"], "'213'", &[]),
        (["5", "0x10", "3"], "'213'", &[]),
        (["5", "0x1p3", "3"], "'213'", &[]),
        (["5", "0x1p-2", "3"], "'132'", &[]),
        (
            ["5", "1.5x", "3"],
            "'132'",
            &["Line 2: score '1.5x' partially read as '1.5'"],
        ),
        (
            ["5", "5e", "3"],
            "'213'",
            &["Line 2: score '5e' partially read as '5'"],
        ),
        (
            ["5", "1e400", "3"],
            "'213'",
            &["Line 2: score '1e400' out of range, read as inf"],
        ),
        (
            ["1e401", "1e400", "-1e400"],
            "'213'",
            &[
                "Line 1: score '1e401' out of range, read as inf",
                "Line 2: score '1e400' out of range, read as inf",
                "Line 3: score '-1e400' out of range, read as -inf",
            ],
        ),
        (
            ["2e-400", "1e-400", "-1e-400"],
            "'321'",
            &[
                "Line 1: score '2e-400' out of range, read as 0",
                "Line 2: score '1e-400' out of range, read as 0",
                "Line 3: score '-1e-400' out of range, read as -0",
            ],
        ),
        // A subnormal score is in range.
        (["2e-310", "1e-310", "0"], "'123'", &[]),
    ];
    for &(scores, relstring, warnings) in cases {
        let run = write("scores.run", scores_run(scores));
        let out = eval(&["-q", "-m", "relstring"], &qrels, &run);
        assert_eq!(
            rows(&out),
            [format!("relstring 1 {relstring}")],
            "{scores:?}"
        );
        let warnings: Vec<_> = warnings
            .iter()
            .map(|warning| format!("Warning in run file '{}': {warning}", run.display()))
            .collect();
        assert_eq!(
            stderr(&out).lines().collect::<Vec<_>>(),
            warnings,
            "{scores:?}"
        );
    }
}

#[test]
fn nan_and_scores_without_a_number_are_refused() {
    let qrels = write("abc.qrels", "1 0 a 1\n1 0 b 2\n1 0 c 3\n");
    for score in ["nan", "abc"] {
        let run = write("scores.run", scores_run(["5", score, "3"]));
        let out = eval(&["-m", "relstring"], &qrels, &run);
        assert_refused(
            &out,
            &format!("Line 2: invalid float for field 'sim': '{score}'"),
        );
    }
}

/// A run that gives documents a, b and c the scores `scores`.
fn scores_run(scores: [&str; 3]) -> String {
    ["a", "b", "c"]
        .iter()
        .zip(scores)
        .enumerate()
        .map(|(i, (docno, score))| format!("1 Q0 {docno} {} {score} r\n", i + 1))
        .collect()
}

#[test]
fn a_fractional_grade_scores_as_its_leading_integer_with_a_warning() {
    // a is graded 1.5 and b 0.9. Read as 1 and 0, only a is relevant, and a is at rank 2:
    // AP = 1/2. Rounding would make b relevant too and give AP = 1.
    let fractional = write("fractional.qrels", "t 0 a 1.5\nt 0 b 0.9\nt 0 c 0\n");
    let integer = write("integer.qrels", "t 0 a 1\nt 0 b 0\nt 0 c 0\n");
    let run = write("grades.run", "t Q0 b 1 3 r\nt Q0 a 2 2 r\nt Q0 c 3 1 r\n");
    for (format, measure) in [("qrels", "map"), ("qrels_jg", "map_avgjg")] {
        let options = ["-R", format, "-m", measure];
        let (read, whole) = (
            eval(&options, &fractional, &run),
            eval(&options, &integer, &run),
        );
        assert_eq!(summary(&read, measure), "0.5000");
        assert_eq!(read.stdout, whole.stdout, "{format}");
        let prefix = format!("Warning in {format} file '{}'", fractional.display());
        assert_eq!(
            stderr(&read).lines().collect::<Vec<_>>(),
            [
                format!("{prefix}: Line 1: value '1.5' partially read as '1'"),
                format!("{prefix}: Line 2: value '0.9' partially read as '0'"),
            ]
        );
        assert!(whole.stderr.is_empty(), "{}", stderr(&whole));
    }
}

#[test]
fn grades_127_and_minus_127_are_scored() {
    // d1 is relevant at rank 1 in both topics, at level 1 and then at level 127, as in
    // trec_eval's test_grade-range.sh.
    let qrels = write(
        "edge.qrels",
        "1 0 d1 1\n1 0 d2 0\n2 0 d1 127\n2 0 d2 -127\n",
    );
    let run = write(
        "grades.run",
        "1 Q0 d1 1 2 r\n1 Q0 d2 2 1 r\n2 Q0 d1 1 2 r\n2 Q0 d2 2 1 r\n",
    );
    assert_eq!(
        summary(&eval(&["-m", "map"], &qrels, &run), "map"),
        "1.0000"
    );
    let out = eval(&["-R", "qrels_jg", "-m", "map_avgjg"], &qrels, &run);
    assert_eq!(summary(&out, "map_avgjg"), "1.0000");
}

#[test]
fn with_floats_file_scores_like_qrels_test() {
    // The Makefile check: `trec_eval test/qrels.test.with-floats test/results.test` must print
    // `test/out.test`, the output for `test/qrels.test`.
    let results = test_data("results.test");
    let floats = eval(&["-q"], &test_data("qrels.test.with-floats"), &results);
    let integers = eval(&["-q"], &test_data("qrels.test"), &results);
    assert_eq!(rows(&floats), rows(&integers));
}

/// A repeated document is refused only where trec_eval refuses it: in a topic that is scored,
/// within `-M`, when a requested measure reads the ranking. The expected values are trec_eval's.
mod scored_duplicates {
    use super::common::{eval, fixture, stderr, summary, write};
    use std::path::PathBuf;
    use std::process::Output;

    /// `topics.run` with d2 at rank 3 of topic 1 again, in place of d3.
    const REPEAT_AT_RANK_3: &str =
        "1 Q0 d1 1 3 r\n1 Q0 d2 2 2 r\n1 Q0 d2 3 1 r\n2 Q0 d2 1 2 r\n2 Q0 d1 2 1 r\n";

    /// The fixture `name` with `lines` added.
    fn with_lines(name: &str, lines: &str) -> PathBuf {
        write(
            name,
            std::fs::read_to_string(fixture(name)).unwrap() + lines,
        )
    }

    /// Checks for exit status 4, trec_eval's status for a repeat, and returns stderr.
    fn refusal(output: &Output) -> String {
        assert_eq!(output.status.code(), Some(4), "{output:?}");
        stderr(output)
    }

    #[test]
    fn a_repeat_in_a_run_topic_absent_from_the_qrels_is_not_checked() {
        // Topic 9 is never scored. Topic 1 ranks d1, d2, d3: AP = (1/1 + 2/3) / 2 = 0.8333.
        // Topic 2 ranks d2 first: AP = 1. map = 0.9167.
        let run = with_lines("topics.run", "9 Q0 z 1 2 r\n9 Q0 z 2 1 r\n");
        let out = eval(&["-m", "map"], &fixture("topics.qrels"), &run);
        assert_eq!(summary(&out, "map"), "0.9167");
    }

    #[test]
    fn a_repeat_beyond_max_docs_is_not_checked() {
        // With -M 2, topic 1 ranks d1, d2: AP = (1/1) / 2 = 0.5, and map = (0.5 + 1) / 2.
        let (qrels, run) = (
            fixture("topics.qrels"),
            write("repeat.run", REPEAT_AT_RANK_3),
        );
        assert_eq!(
            summary(&eval(&["-M", "2", "-m", "map"], &qrels, &run), "map"),
            "0.7500"
        );
        let out = eval(
            &["-R", "qrels_jg", "-M", "2", "-m", "map_avgjg"],
            &qrels,
            &run,
        );
        assert_eq!(summary(&out, "map_avgjg"), "0.7500");
    }

    #[test]
    fn a_repeat_within_max_docs_is_refused() {
        let (qrels, run) = (
            fixture("topics.qrels"),
            write("repeat.run", REPEAT_AT_RANK_3),
        );
        let message = format!(
            "Error in run file '{}': duplicate document 'd2' for query '1'",
            run.display()
        );
        assert!(refusal(&eval(&["-m", "map"], &qrels, &run)).contains(&message));
        let out = eval(&["-R", "qrels_jg", "-m", "map_avgjg"], &qrels, &run);
        assert!(refusal(&out).contains(&message));
    }

    #[test]
    fn measures_that_read_no_ranking_skip_the_check() {
        // num_q and runid form no ranking in trec_eval, which prints num_q 2 for both files.
        let (qrels, run) = (
            fixture("topics.qrels"),
            write("repeat.run", REPEAT_AT_RANK_3),
        );
        assert_eq!(summary(&eval(&["-m", "num_q"], &qrels, &run), "num_q"), "2");
        let out = eval(&["-m", "runid", "-m", "num_q"], &qrels, &run);
        assert_eq!(summary(&out, "runid"), "r");
        let qrels = with_lines("topics.qrels", "1 0 d1 0\n");
        let out = eval(&["-m", "num_q"], &qrels, &fixture("topics.run"));
        assert_eq!(summary(&out, "num_q"), "2");
    }

    #[test]
    fn a_repeated_judgment_is_refused_only_in_a_scored_topic() {
        let qrels = with_lines("topics.qrels", "1 0 d1 0\n");
        let message = format!(
            "Error in qrels file '{}': duplicate document 'd1' for query '1'",
            qrels.display()
        );
        assert!(refusal(&eval(&["-m", "map"], &qrels, &fixture("topics.run"))).contains(&message));
        // Topic 3 has no results. Without -c it is not scored, and map is that of topics 1 and 2.
        // trec_eval 10.x exits 4 here because topic 3 has no results, as docs/cli.md says.
        let qrels = with_lines("topics.qrels", "3 0 x 1\n3 0 x 0\n");
        let out = eval(&["-m", "map"], &qrels, &fixture("topics.run"));
        assert_eq!(summary(&out, "map"), "0.9167");
        // With -c topic 3 is scored, and trec_eval refuses the repeat too.
        let out = eval(&["-c", "-m", "map"], &qrels, &fixture("topics.run"));
        assert!(refusal(&out).contains("duplicate document 'x' for query '3'"));
    }
}
