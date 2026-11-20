use std::path::PathBuf;
use std::process::Command;

fn evaluate(binary: &std::path::Path, level: i64, run: &str) -> (String, String) {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/negative_11pt");
    let output = Command::new(binary)
        .args([&format!("-l{level}"), "-m", "11pt_avg", "-m", "num_rel_ret"])
        .arg(fixtures.join("topic.qrels"))
        .arg(fixtures.join(format!("{run}.run")))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let value = |name| {
        stdout
            .lines()
            .find_map(|line| {
                let fields: Vec<_> = line.split_whitespace().collect();
                (fields[0] == name && fields[1] == "all").then(|| fields[2].to_owned())
            })
            .unwrap()
    };
    (value("num_rel_ret"), value("11pt_avg"))
}

#[test]
fn negative_levels_match_the_c_reverse_scan_for_unjudged_documents() {
    // Columns are -l-2, -l-1, -l0 and -l1, as printed by trec_eval.
    let cases = [
        ("xa", ["0.3636", "0.3636", "0.3636", "0.5000"]),
        ("ax", ["0.3636", "0.3636", "0.7273", "1.0000"]),
        ("ua", ["0.3636", "0.3636", "0.3636", "0.5000"]),
        ("au", ["0.3636", "0.7273", "0.7273", "1.0000"]),
        ("xua", ["0.2424", "0.2424", "0.2424", "0.3333"]),
        ("aux", ["0.2424", "0.2424", "0.7273", "1.0000"]),
        ("xu", ["0.0000", "0.0000", "0.0000", "0.0000"]),
        ("ab", ["1.0000", "1.0000", "1.0000", "1.0000"]),
    ];
    for (run, values) in cases {
        for (level, expected) in [-2, -1, 0, 1].into_iter().zip(values) {
            let relevant = if run == "ab" && level <= 0 {
                "2"
            } else if run == "xu" {
                "0"
            } else {
                "1"
            };
            let actual = evaluate(
                std::path::Path::new(env!("CARGO_BIN_EXE_te-rust")),
                level,
                run,
            );
            assert_eq!(
                actual,
                (relevant.to_owned(), expected.to_owned()),
                "{run}, -l{level}"
            );
            if let Some(dir) = std::env::var_os("TREC_EVAL_DIR") {
                let reference = evaluate(&PathBuf::from(dir).join("trec_eval"), level, run);
                assert_eq!(actual, reference, "trec_eval: {run}, -l{level}");
            }
        }
    }
}
