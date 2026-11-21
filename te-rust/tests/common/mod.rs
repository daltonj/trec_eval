//! Running te-rust from the integration tests, and the inputs they share.
//!
//! `tests/fixtures` holds the inputs that tests in more than one place read:
//!
//! - `ab.qrels` judges a relevant and b nonrelevant in topic 1, and `ab.run` ranks a, then b.
//! - `graded.qrels` grades a 2, b 0, c 1 and d 3 in topic t, and `graded.run` ranks d, x
//!   (unjudged), a, b, c.
//! - `six-relevant.qrels` judges d1 to d6 relevant in topic q1, and `six-relevant.run` ranks
//!   them in that order.
//! - `topics.qrels` judges d1 (1), d2 (0) and d3 (2) in topic 1, and d1 (0) and d2 (1) in
//!   topic 2. `topics.run` ranks d1, d2, d3 for topic 1 and d2, d1 for topic 2.
//!
//! A test writes any other input with [`write`].
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn te_rust() -> Command {
    Command::new(env!("CARGO_BIN_EXE_te-rust"))
}

pub fn trec_eval_bin() -> PathBuf {
    let dir = std::env::var_os("TREC_EVAL_DIR").unwrap_or_else(|| "trec_eval".into());
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(dir)
        .join("trec_eval")
}

/// Runs te-rust with `options` on `qrels` and `run`.
pub fn eval(options: &[&str], qrels: &Path, run: &Path) -> Output {
    te_rust()
        .args(options)
        .arg(qrels)
        .arg(run)
        .output()
        .expect("failed to run te-rust")
}

pub fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// A file of trec_eval's `test` directory, beside the reference binary.
pub fn test_data(name: &str) -> PathBuf {
    trec_eval_bin().with_file_name("test").join(name)
}

/// Writes `contents` to a new file whose name ends with `name`.
pub fn write(name: &str, contents: impl AsRef<[u8]>) -> PathBuf {
    static FILES: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "{}-{}",
        env!("CARGO_CRATE_NAME"),
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("failed to create the directory for test files");
    let path = dir.join(format!("{}-{name}", FILES.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(&path, contents).expect("failed to write a test file");
    path
}

/// The rows that te-rust printed, each as `measure topic value`, once it has succeeded.
pub fn rows(output: &Output) -> Vec<String> {
    assert!(
        output.status.success(),
        "te-rust failed: {}",
        stderr(output)
    );
    normalized_rows(&String::from_utf8_lossy(&output.stdout))
}

/// The stored rows from trec_eval's `test` directory.
pub fn reference_rows(name: &str) -> Vec<String> {
    normalized_rows(&std::fs::read_to_string(test_data(name)).unwrap())
}

fn normalized_rows(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

/// The value that te-rust printed for `measure` on the `all` row.
pub fn summary(output: &Output, measure: &str) -> String {
    let label = format!("{measure} all ");
    rows(output)
        .iter()
        .find_map(|row| row.strip_prefix(&label).map(str::to_string))
        .unwrap_or_else(|| panic!("no summary row for {measure}"))
}

pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// Checks that te-rust failed, printed no rows, and wrote `message` to stderr.
pub fn assert_refused(output: &Output, status: i32, message: &str) {
    let stderr = stderr(output);
    assert_eq!(output.status.code(), Some(status), "{stderr}");
    assert!(output.stdout.is_empty(), "te-rust printed rows: {stderr}");
    assert!(stderr.contains(message), "{message:?} is not in {stderr:?}");
}
