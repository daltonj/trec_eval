use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/duplicate-measure-rows")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn te_rust(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(args)
        .output()
        .expect("failed to run te-rust")
}

/// Printed rows as (measure, topic, value), in output order.
fn rows(output: &Output) -> Vec<(String, String, String)> {
    assert!(
        output.status.success(),
        "te-rust failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            let mut fields = line.split_whitespace();
            let mut next = || fields.next().unwrap_or_default().to_string();
            (next(), next(), next())
        })
        .collect()
}

/// The printed value of `measure` on the `all` row.
fn summary(output: &Output, measure: &str) -> String {
    rows(output)
        .into_iter()
        .find(|(name, topic, _)| name == measure && topic == "all")
        .unwrap_or_else(|| panic!("no summary row for {measure}"))
        .2
}

fn names(extra: &[&str]) -> Vec<String> {
    let (qrels, run) = (fixture("topic.qrels"), fixture("topic.run"));
    let mut args = extra.to_vec();
    args.extend([qrels.as_str(), run.as_str()]);
    rows(&te_rust(&args))
        .into_iter()
        .map(|row| format!("{} {}", row.0, row.1))
        .collect()
}

fn assert_each_once(extra: &[&str]) {
    let all = names(extra);
    let distinct: std::collections::HashSet<_> = all.iter().collect();
    assert_eq!(all.len(), distinct.len(), "{extra:?}: {all:?}");
}

#[test]
fn repeated_measure_prints_once() {
    assert_eq!(names(&["-m", "map", "-m", "map"]), ["map all"]);
    // All six documents are relevant and retrieved, so the single row is 1.
    let (qrels, run) = (fixture("topic.qrels"), fixture("topic.run"));
    let out = te_rust(&["-m", "map", "-m", "map", &qrels, &run]);
    assert_eq!(summary(&out, "map"), "1.0000");
}

#[test]
fn group_and_member_print_once() {
    assert_each_once(&["-m", "official", "-m", "map"]);
    assert_each_once(&["-m", "map", "-m", "official"]);
    for args in [
        ["-m", "official", "-m", "P.5"],
        ["-m", "P.5", "-m", "official"],
    ] {
        let rows = names(&args);
        assert_eq!(
            rows.iter()
                .filter(|name| name.starts_with("P_"))
                .collect::<Vec<_>>(),
            ["P_5 all"]
        );
        assert_each_once(&args);
    }
}

#[test]
fn aliases_and_reordered_lists_print_once() {
    assert_each_once(&["-m", "P.5", "-m", "P_5", "-m", "P@5"]);
    assert_each_once(&["-m", "P.5,10", "-m", "P.10,5"]);
}

#[test]
fn a_request_covered_by_an_earlier_one_adds_nothing() {
    // trec_eval reads only the first -m P here, which prints P_2 and P_5.
    assert_eq!(names(&["-m", "P.2,5", "-m", "P.5"]), ["P_2 all", "P_5 all"]);
}

#[test]
fn per_topic_rows_print_once() {
    assert_each_once(&["-q", "-m", "map", "-m", "map"]);
}

#[test]
fn one_list_with_colliding_row_names_is_refused() {
    // Distinct multiples that both print as 1.00 cannot be distinguished by row name.
    assert!(!te_rust(&[
        "-m",
        "Rprec_mult.0.999,1.001",
        &fixture("topic.qrels"),
        &fixture("topic.run")
    ])
    .status
    .success());
}

#[test]
fn first_explicit_parameter_list_wins() {
    for (first, second, expected) in [
        ("P.2", "P.3", vec!["P_2 all"]),
        ("P.2,5", "P.5,10", vec!["P_2 all", "P_5 all"]),
        ("P.2", "P.2,3", vec!["P_2 all"]),
        (
            "Rprec_mult.0.219",
            "Rprec_mult.0.221,1",
            vec!["Rprec_mult_0.22 all"],
        ),
        ("P", "P.2", vec!["P_2 all"]),
    ] {
        assert_eq!(names(&["-m", first, "-m", second]), expected);
    }
}
