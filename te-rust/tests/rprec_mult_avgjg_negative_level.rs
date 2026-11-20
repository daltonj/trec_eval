use std::path::PathBuf;
use std::process::Command;

#[test]
fn reverse_scan_stops_when_the_relevant_count_reaches_zero() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rprec-mult-avgjg-negative-level");
    for level in ["-1", "-2"] {
        let out = Command::new(env!("CARGO_BIN_EXE_te-rust"))
            .args([
                "-R",
                "qrels_jg",
                &format!("-l{level}"),
                "-m",
                "Rprec_mult_avgjg.0.5,1,2,4",
            ])
            .arg(fixtures.join("one.qrels"))
            .arg(fixtures.join("axy.run"))
            .output()
            .unwrap();
        assert!(out.status.success(), "-l{level}: {out:?}");
        let stdout = String::from_utf8(out.stdout).unwrap();
        let rows: Vec<_> = stdout
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .collect();
        assert_eq!(
            rows,
            vec![
                vec!["Rprec_mult_avgjg_0.50", "all", "0.0000"],
                vec!["Rprec_mult_avgjg_1.00", "all", "0.0000"],
                vec!["Rprec_mult_avgjg_2.00", "all", "0.0000"],
                vec!["Rprec_mult_avgjg_4.00", "all", "0.2500"],
            ],
            "-l{level}"
        );
    }
}
