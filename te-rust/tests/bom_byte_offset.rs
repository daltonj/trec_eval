use std::path::PathBuf;
use std::process::Command;

#[test]
fn utf8_error_offsets_include_the_first_line_bom() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/line-bytes");
    for (name, bytes, format, column) in [
        (
            "bad.qrels",
            b"\xef\xbb\xbf1 0 d\xe9 1\n".as_slice(),
            "qrels",
            9,
        ),
        (
            "bad-jg.qrels",
            b"\xef\xbb\xbf1 0 d\xe9 1\n".as_slice(),
            "qrels_jg",
            9,
        ),
        (
            "bad.run",
            b"\xef\xbb\xbf1 Q0 d\xe9 1 3 r\n".as_slice(),
            "qrels",
            10,
        ),
    ] {
        let bad = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("bom-{name}"));
        std::fs::write(&bad, bytes).unwrap();
        let (qrels, run) = if name.ends_with(".run") {
            (fixtures.join("abc.qrels"), bad)
        } else {
            (bad, fixtures.join("abc.run"))
        };
        let out = Command::new(env!("CARGO_BIN_EXE_te-rust"))
            .args(["-R", format, "-m", "map"])
            .arg(qrels)
            .arg(run)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{name}");
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(
            stderr.contains(&format!("Line 1: invalid UTF-8 at byte {column}")),
            "{name}: {stderr}"
        );
    }
}
