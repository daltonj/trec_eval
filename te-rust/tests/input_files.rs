//! File errors and deferred duplicate checks at the CLI boundary.

mod common;

use common::{assert_refused, eval, rows, stderr, summary, write};

#[test]
fn file_warnings_name_the_format_and_line() {
    let qrels = write("grade.qrels", "1 0 a 1.5\n");
    let run = write("scores.run", "1 Q0 a 1 1e400x r\n");
    for (kind, measure) in [("qrels", "map"), ("qrels_jg", "map_avgjg")] {
        let out = eval(&["-R", kind, "-m", measure], &qrels, &run);
        assert_eq!(summary(&out, measure), "1.0000");
        assert_eq!(
            stderr(&out),
            format!(
                "Warning in {kind} file '{}': Line 1: value '1.5' partially read as '1'\n\
             Warning in run file '{}': Line 1: score '1e400x' out of range, read as inf\n\
             Warning in run file '{}': Line 1: score '1e400x' partially read as '1e400'\n",
                qrels.display(),
                run.display(),
                run.display()
            )
        );
    }
}

#[test]
fn repeated_documents_name_the_source_and_exit_4() {
    let good_qrels = write("one.qrels", "1 0 a 1\n");
    let good_run = write("one.run", "1 Q0 a 1 2 r\n");
    let repeated_qrels = write("repeat.qrels", "1 0 a 1\n1 0 a 0\n");
    let repeated_run = write("repeat.run", "1 Q0 a 1 2 r\n1 Q0 a 2 1 r\n");
    for (kind, measure) in [("qrels", "map"), ("qrels_jg", "map_avgjg")] {
        assert_refused(
            &eval(&["-R", kind, "-m", measure], &good_qrels, &repeated_run),
            4,
            &format!(
                "Error in run file '{}': Line 2: duplicate document 'a' for query '1', \
                 first at line 1",
                repeated_run.display()
            ),
        );
        // The retained prefix is unique; the discarded repeat is not checked.
        assert_eq!(
            summary(
                &eval(
                    &["-R", kind, "-M", "1", "-m", measure],
                    &good_qrels,
                    &repeated_run
                ),
                measure
            ),
            "1.0000"
        );
    }
    assert_refused(
        &eval(&["-m", "map"], &repeated_qrels, &good_run),
        4,
        &format!(
            "Error in qrels file '{}': Line 2: duplicate document 'a' for query '1', \
             first at line 1",
            repeated_qrels.display()
        ),
    );
    // Neither measure reads a ranking or checks duplicates, even in the qrels.
    let out = eval(
        &["-m", "num_q", "-m", "runid"],
        &repeated_qrels,
        &repeated_run,
    );
    assert_eq!(rows(&out), ["runid all r", "num_q all 1"]);
}

#[test]
fn duplicates_in_unscored_topics_are_not_checked() {
    let qrels = write("missing.qrels", "1 0 a 1\n2 0 b 1\n2 0 b 0\n");
    let run = write("extra.run", "1 Q0 a 1 2 r\n3 Q0 x 1 2 r\n3 Q0 x 2 1 r\n");
    assert_eq!(
        summary(&eval(&["-m", "map"], &qrels, &run), "map"),
        "1.0000"
    );
    assert_refused(
        &eval(&["-c", "-m", "map"], &qrels, &run),
        4,
        "duplicate document 'b' for query '2'",
    );
}
