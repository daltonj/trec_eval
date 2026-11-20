use std::path::PathBuf;
use std::process::Command;

#[test]
fn integer_options_skip_leading_c_locale_whitespace() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/argument-prefix");
    for (option, value, measure, expected) in [
        ("-l", " 2", "map", "0.5000"),
        ("-M", "\t2", "map", "0.2500"),
        ("-N", " 10", "utility.0,0,0,1", "7.0000"),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_te-rust"))
            .args([option, value, "-m", measure])
            .arg(fixtures.join("graded.qrels"))
            .arg(fixtures.join("graded.run"))
            .output()
            .unwrap();
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(out.status.success(), "{option}: {stderr}");
        assert!(stderr.is_empty(), "{option}: {stderr}");
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert_eq!(stdout.split_whitespace().last(), Some(expected), "{option}");
    }
}
