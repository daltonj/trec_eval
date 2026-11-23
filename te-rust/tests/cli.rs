//! The te-rust command: its options, exit statuses and output.

mod common;

/// Exit statuses and validation precedence. te-rust validates measure parameters
/// before reading files; trec_eval validates them afterward.
mod exit_status {
    use super::common::{assert_refused, eval, fixture, te_rust, write};
    use std::path::PathBuf;

    fn input(name: &str) -> PathBuf {
        let text = match name {
            "good.qrels" => return fixture("ab.qrels"),
            "good.run" => return fixture("ab.run"),
            "absent.qrels" | "absent.run" => return fixture(name),
            "malformed.qrels" => "1 0 a\n",
            "malformed.run" => "1 Q0 a 1\n",
            "repeat.qrels" => "1 0 a 1\n1 0 a 0\n",
            _ => panic!("no input {name}"),
        };
        write(name, text)
    }

    fn status(options: &[&str], qrels: &str, run: &str) -> Option<i32> {
        eval(options, &input(qrels), &input(run)).status.code()
    }

    #[test]
    fn input_errors_exit_2() {
        let map = &["-m", "map"][..];
        let jg = &["-R", "qrels_jg", "-m", "map_avgjg"][..];
        for (options, qrels, run) in [
            (map, "good.qrels", "malformed.run"),
            (jg, "malformed.qrels", "good.run"),
            (jg, "repeat.qrels", "good.run"),
        ] {
            assert_eq!(
                status(options, qrels, run),
                Some(2),
                "{options:?} {qrels} {run}"
            );
        }
    }

    #[test]
    fn an_unusable_measure_parameter_exits_2() {
        // Parameter validation precedes file access.
        assert_refused(
            &eval(&["-m", "P.x"], &input("absent.qrels"), &input("absent.run")),
            2,
            "invalid integer cutoff 'x'",
        );
    }

    #[test]
    fn an_illegal_measure_name_exits_1_before_the_files_are_read() {
        // All names are checked before parameters, even a bad parameter in an earlier request.
        assert_refused(
            &eval(
                &["-m", "P.x", "-m", "nosuch"],
                &input("absent.qrels"),
                &input("absent.run"),
            ),
            1,
            "unknown measure 'nosuch'",
        );
    }

    #[test]
    fn a_missing_run_filename_exits_1() {
        let out = te_rust()
            .args(["-m", "map"])
            .arg(input("good.qrels"))
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1));
    }

    #[test]
    fn the_qrels_are_checked_before_the_run() {
        let map = &["-m", "map"];
        assert_refused(
            &eval(map, &input("absent.qrels"), &input("absent.run")),
            2,
            "absent.qrels",
        );
        let qrels = input("malformed.qrels");
        assert_refused(
            &eval(map, &qrels, &input("malformed.run")),
            2,
            &qrels.to_string_lossy(),
        );
    }

    #[test]
    fn the_judgment_format_is_checked_before_the_run() {
        let options = &["-R", "nosuch"];
        assert_refused(
            &eval(options, &input("good.qrels"), &input("absent.run")),
            2,
            "nosuch",
        );
    }

    #[test]
    fn a_repeated_judgment_is_found_after_the_run_is_read() {
        let (qrels, run) = (input("repeat.qrels"), input("malformed.run"));
        assert_refused(
            &eval(&["-m", "map"], &qrels, &run),
            2,
            &run.to_string_lossy(),
        );
    }
}

/// Failed writes to standard output. trec_eval ignores them and exits 0; te-rust reports them.
#[cfg(unix)]
mod output {
    use super::common::{fixture, stderr, te_rust, write};
    use std::fs::File;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{Command, Output};

    // trec_eval's status when it cannot print a measure.
    const EXIT_OUTPUT: i32 = 8;

    /// Runs te-rust under `sh` with a file size limit of 0 and SIGXFSZ ignored, so that every
    /// write to a regular file fails with EFBIG, as writes fail on a full disk.
    fn te_rust_with_no_room(args: &[&str]) -> Command {
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg("trap '' XFSZ; ulimit -f 0; exec \"$0\" \"$@\"")
            .arg(env!("CARGO_BIN_EXE_te-rust"))
            .args(args);
        command
    }

    fn file(name: &str) -> File {
        File::options().write(true).open(write(name, "")).unwrap()
    }

    fn assert_write_failure(output: &Output) {
        let stderr = stderr(output);
        assert!(!stderr.contains("panicked"), "{stderr}");
        assert!(
            stderr.contains("te-rust: cannot write to standard output"),
            "{stderr}"
        );
        assert_eq!(
            output.status.code(),
            Some(EXIT_OUTPUT),
            "{:?}",
            output.status
        );
    }

    #[test]
    fn closed_stdout_ends_with_sigpipe_and_no_panic() {
        use std::os::unix::process::CommandExt;

        let mut command = te_rust();
        command
            .args(["-m", "map"])
            .arg(fixture("ab.qrels"))
            .arg(fixture("ab.run"));
        // Create the pipe in the child so concurrent spawns cannot inherit its reader.
        // SAFETY: only async-signal-safe libc calls run between fork and exec.
        unsafe {
            command.pre_exec(|| {
                let mut pipe = [0; 2];
                if libc::pipe(pipe.as_mut_ptr()) != 0
                    || libc::close(pipe[0]) != 0
                    || libc::dup2(pipe[1], libc::STDOUT_FILENO) < 0
                    || libc::close(pipe[1]) != 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let output = command.output().unwrap();
        assert!(!stderr(&output).contains("panicked"), "{}", stderr(&output));
        assert_eq!(
            output.status.signal(),
            Some(libc::SIGPIPE),
            "{:?}",
            output.status
        );
    }

    #[test]
    fn failed_result_write_exits_with_a_message() {
        let (qrels, run) = (fixture("ab.qrels"), fixture("ab.run"));
        let args = ["-q", qrels.to_str().unwrap(), run.to_str().unwrap()];
        let output = te_rust_with_no_room(&args)
            .stdout(file("results.out"))
            .output()
            .unwrap();
        assert_write_failure(&output);
    }

    #[test]
    fn failed_help_write_exits_with_a_message() {
        for args in [
            &["--help"][..],
            &["--version"],
            &["--help-measures"],
            &["--help-measure", "map"],
        ] {
            let output = te_rust_with_no_room(args)
                .stdout(file("help.out"))
                .output()
                .unwrap();
            assert_write_failure(&output);
        }
    }

    #[test]
    fn a_lost_write_error_message_keeps_status_8() {
        // With stderr on the same full disk, the message is lost but the status is not.
        let (qrels, run) = (fixture("ab.qrels"), fixture("ab.run"));
        let both = file("both.out");
        let args = [qrels.to_str().unwrap(), run.to_str().unwrap()];
        let output = te_rust_with_no_room(&args)
            .stdout(both.try_clone().unwrap())
            .stderr(both)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(EXIT_OUTPUT),
            "{:?}",
            output.status
        );
    }
}

/// Representative differences from trec_eval documented in docs/cli.md.
mod documented_differences {
    use super::common::{assert_refused, te_rust, test_data};
    use std::fs::File;
    use std::process::Output;

    fn eval_test_data(options: &[&str], run: &str) -> Output {
        let mut command = te_rust();
        command.args(options).arg(test_data("qrels.test")).arg(run);
        if run == "-" {
            command.stdin(File::open(test_data("results.test")).unwrap());
        }
        command.output().unwrap()
    }

    #[test]
    fn listed_trec_eval_features_are_refused() {
        for (options, status, message) in [
            (&["-T", "trec_results"][..], 1, "unexpected argument '-T'"),
            (&["-m", "prefs_simp"], 1, "unknown measure 'prefs_simp'"),
            (
                &["-R", "prefs"],
                2,
                "Unsupported relevance judgments format 'prefs'",
            ),
        ] {
            assert_refused(
                &eval_test_data(options, test_data("results.test").to_str().unwrap()),
                status,
                message,
            );
        }
        // trec_eval reads the run from stdin when its name is "-".
        assert_refused(
            &eval_test_data(&["-m", "map"], "-"),
            2,
            "Error parsing run file '-'",
        );
        assert_refused(
            &te_rust().arg("-v").output().unwrap(),
            1,
            "unexpected argument '-v'",
        );
    }

    #[test]
    fn rel_format_help_lists_only_supported_formats() {
        let output = te_rust().arg("--help").output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout
            .lines()
            .find(|line| line.contains("--rel-format"))
            .expect("no -R line in --help");
        assert!(line.contains("qrels_jg"), "{line}");
        assert!(!line.contains("prefs"), "{line}");
    }
}

/// Which measures te-rust evaluates for its `-m` options, and the order of their rows.
mod measure_requests {
    use super::common::{eval, fixture, rows};

    #[test]
    fn requests_print_once_in_trec_eval_order() {
        let out = eval(
            &[
                "-q", "-m", "ndcg_p", "-m", "map", "-m", "num_q", "-m", "map",
            ],
            &fixture("ab.qrels"),
            &fixture("ab.run"),
        );
        assert_eq!(
            rows(&out),
            [
                "map 1 1.0000",
                "ndcg_p 1 1.0000",
                "num_q all 1",
                "map all 1.0000",
                "ndcg_p all 1.0000"
            ]
        );
    }

    #[test]
    fn group_defaults_yield_to_an_explicit_member_list() {
        for options in [
            ["-m", "official", "-m", "P.2"],
            ["-m", "P.2", "-m", "official"],
        ] {
            let out = eval(&options, &fixture("ab.qrels"), &fixture("ab.run"));
            let precision: Vec<_> = rows(&out)
                .into_iter()
                .filter(|r| r.starts_with("P_"))
                .collect();
            assert_eq!(precision, ["P_2 all 0.5000"]);
        }
    }
}

/// Gain maps supplied with --global-gains.
mod global_gains {
    use super::common::{assert_refused, eval, fixture};

    #[test]
    fn invalid_gains_and_repeated_levels_are_refused() {
        let (qrels, run) = (fixture("topics.qrels"), fixture("topics.run"));
        let out = eval(&["--global-gains", "1=x", "-m", "ndcg"], &qrels, &run);
        assert_refused(&out, 1, "invalid gain 'x' for relevance level 1");
        let out = eval(&["--global-gains", "1=2,1=5", "-m", "ndcg"], &qrels, &run);
        assert_refused(&out, 1, "relevance level 1 is given more than once");
    }
}

/// The integer options -l, -M and -N and integer cutoffs, which trec_eval reads with
/// `str_to_long`: C-locale whitespace, a sign and digits, with a warning for what follows them.
mod integer_options {
    use super::common::{assert_refused, eval, rows, stderr, write};
    use std::process::Output;

    /// b (grade 2) is at rank 2 and a (grade 1) at rank 3.
    fn two_grades(options: &[&str]) -> Output {
        let qrels = write("two-grades.qrels", "1 0 a 1\n1 0 b 2\n1 0 c 0\n");
        let run = write(
            "two-grades.run",
            "1 Q0 c 1 3 r\n1 Q0 b 2 2 r\n1 Q0 a 3 1 r\n",
        );
        eval(options, &qrels, &run)
    }

    #[test]
    fn cutoff_warnings_name_each_measure_request() {
        let out = two_grades(&["-m", "P.1.5", "-m", "recall.5x"]);
        assert_eq!(rows(&out), ["P_1 all 0.0000", "recall_5 all 1.0000"]);
        assert_eq!(
            stderr(&out),
            "te-rust: Warning, measure 'P.1.5': value '1.5' partially read as '1'\n\
             te-rust: Warning, measure 'recall.5x': value '5x' partially read as '5'\n"
        );
    }

    #[test]
    fn relevance_level_and_document_counts_read_their_leading_integer_with_a_warning() {
        for (option, value, read, measure, expected) in [
            // b alone is relevant, at rank 2.
            ("-l", "2.5", "2", "map", "map all 0.5000"),
            // Only b is retained; AP = (1/2) / 2.
            ("-M", "2x", "2", "map", "map all 0.2500"),
            // Nonretrieved nonrelevant = 10 - 3 - (2 - 2).
            (
                "-N",
                "10x",
                "10",
                "utility.0,0,0,1",
                "utility_0,0,0,1 all 7.0000",
            ),
        ] {
            let out = two_grades(&[option, value, "-m", measure]);
            assert_eq!(rows(&out), [expected]);
            assert!(stderr(&out).contains(&format!("value '{value}' partially read as '{read}'")));
        }
    }

    #[test]
    fn integer_options_skip_leading_c_locale_whitespace() {
        for (option, value, measure, row) in [
            // Only b is relevant at level 2, at rank 2: AP = 1/2.
            ("-l", " 2", "map", "map all 0.5000"),
            // Truncation keeps c and b; b contributes 1/2, divided by two relevant documents.
            ("-M", "\t2", "map", "map all 0.2500"),
        ] {
            let out = two_grades(&[option, value, "-m", measure]);
            assert_eq!(rows(&out), [row], "{option}");
            assert!(out.stderr.is_empty(), "{option}: {}", stderr(&out));
        }
    }

    #[test]
    fn unusable_integer_values_are_refused() {
        // trec_eval: "could not be parsed", and "out of range" for a value beyond a long.
        for (options, status, message) in [
            (
                &["-l", ".5", "-m", "map"][..],
                1,
                "value '.5' could not be parsed",
            ),
            (
                &["-l", "9223372036854775808", "-m", "map"],
                1,
                "value '9223372036854775808' out of range",
            ),
            (&["-M-1", "-m", "map"], 1, "value '-1' is negative"),
        ] {
            assert_refused(&two_grades(options), status, message);
        }
    }
}

/// Relevance levels given with -l, including levels above every judged grade.
mod relevance_level {
    use super::common::{assert_refused, fixture, summary, te_rust};
    use std::process::{Output, Stdio};
    use std::time::{Duration, Instant};

    const LIMIT: Duration = Duration::from_secs(20);

    /// Runs te-rust on the topics fixture, and fails instead of waiting for ever on a level
    /// that does not finish.
    fn te_rust_within_limit(options: &[&str]) -> Output {
        let mut child = te_rust()
            .args(options)
            .arg(fixture("topics.qrels"))
            .arg(fixture("topics.run"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let start = Instant::now();
        while child.try_wait().unwrap().is_none() {
            if start.elapsed() > LIMIT {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("te-rust did not finish within {LIMIT:?}: {options:?}");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        child.wait_with_output().unwrap()
    }

    #[test]
    fn bpref_scores_at_ordinary_and_extreme_levels() {
        // At level 1 the topics score 0.5 and 1, hence 0.75. At level 2 only d3 is
        // relevant, below two nonrelevant documents; both topics then score 0.
        for (level, measure, expected) in [
            ("1", "bpref", "0.7500"),
            ("2", "bpref", "0.0000"),
            ("9223372036854775807", "bpref", "0.0000"),
            ("9223372036854775807", "gm_bpref", "0.0000"),
        ] {
            let out = te_rust_within_limit(&["-l", level, "-m", measure]);
            assert_eq!(summary(&out, measure), expected);
        }
    }

    #[test]
    fn a_negative_level_is_refused_before_the_measures_and_files() {
        let out = te_rust()
            .args(["-l", "-1", "-m", "P.x", "missing.qrels", "missing.run"])
            .output()
            .unwrap();
        assert_refused(&out, 1, "te-rust: Unusable relevance level '-1'");
    }
}

/// Topics that the run lacks: an error, unless -c scores them as empty rankings.
mod complete_set {
    use super::common::{assert_refused, eval, fixture, rows, summary, write};
    use std::path::PathBuf;

    /// A run for topic 3 only, which topics.qrels lacks.
    fn topic_3_run() -> PathBuf {
        write("topic-3.run", "3 Q0 d1 1 1 r\n")
    }

    #[test]
    fn no_shared_topics_require_complete_averaging() {
        let (qrels, run) = (fixture("topics.qrels"), topic_3_run());
        let out = eval(&["-m", "map"], &qrels, &run);
        assert_refused(&out, 4, "no topic has both results and judgments");
        assert_refused(
            &eval(&["-R", "qrels_jg", "-m", "map_avgjg"], &qrels, &run),
            4,
            "no topic has both results and judgments",
        );
    }

    #[test]
    fn complete_set_prints_the_average_over_qrels_topics() {
        let qrels = write("two.qrels", "1 0 a 1\n2 0 b 1\n");
        let run = write("one.run", "1 Q0 a 1 1 r\n");
        for (kind, measure) in [("qrels", "map"), ("qrels_jg", "map_avgjg")] {
            let out = eval(&["-R", kind, "-c", "-m", measure], &qrels, &run);
            assert_eq!(summary(&out, measure), "0.5000");
            let out = eval(&["-R", kind, "-c", "-m", measure], &qrels, &topic_3_run());
            assert_eq!(summary(&out, measure), "0.0000");
        }
    }

    #[test]
    fn no_summary_flag_omits_summary_rows() {
        let out = eval(
            &["-c", "-n", "-m", "map"],
            &fixture("topics.qrels"),
            &topic_3_run(),
        );
        assert_eq!(rows(&out), Vec::<String>::new());
    }
}
