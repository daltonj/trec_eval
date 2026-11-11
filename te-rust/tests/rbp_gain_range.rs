use std::path::PathBuf;
use std::process::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rbp-gain-range")
        .join(name)
}

#[test]
fn normalisation_range_includes_zero_for_both_gain_sources() {
    for (gains, one, both) in [
        ("0=2,1=2", "0.1000", "0.1900"),
        ("0=-4,1=-2", "0.0500", "0.0500"),
        ("0=1,1=2", "0.1000", "0.1450"),
        ("0=-2,1=-1", "0.0500", "0.0500"),
    ] {
        for global in [false, true] {
            let measure = format!("rbp.{gains}");
            let args = if global {
                vec!["--global-gains", gains, "-m", "rbp"]
            } else {
                vec!["-m", &measure]
            };
            for (data, expected) in [("one", one), ("both", both)] {
                let out = Command::new(env!("CARGO_BIN_EXE_te-rust"))
                    .args(&args)
                    .arg(fixture(&format!("{data}.qrels")))
                    .arg(fixture(&format!("{data}.run")))
                    .output()
                    .unwrap();
                assert!(out.status.success(), "{:?}", out);
                let stdout = String::from_utf8(out.stdout).unwrap();
                assert_eq!(
                    stdout.split_whitespace().last(),
                    Some(expected),
                    "{gains}, global={global}, data={data}"
                );
            }
        }
    }
}

#[test]
fn absent_level_keeps_zero_gain_with_an_all_negative_table() {
    let out = Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(["-m", "rbp.0=-4,1=-2"])
        .arg(fixture("one.qrels"))
        .arg(fixture("unjudged.run"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.split_whitespace().last(), Some("0.0450"));
}
