//! The te-rust command: its options, exit statuses and output.

mod common;

/// trec_eval's exit statuses and the order in which it finds problems: an illegal measure name
/// while it reads its options (1), then the qrels and the run (2), the measure parameters (2),
/// and a repeated document while it scores (4). Each status is the one that the C tree in
/// trec_eval/ and v10.0 (f425365) return on the same files.
mod exit_status {
    use super::common::{eval, fixture, stderr, te_rust, write};
    use std::path::{Path, PathBuf};

    /// The input `name`: `good`, `malformed` (too few fields), `empty`, `repeat` (document a
    /// twice in topic 1) or `absent` (no such file), with the extension `.qrels` or `.run`.
    fn input(name: &str) -> PathBuf {
        let text = match name {
            "good.qrels" => return fixture("ab.qrels"),
            "good.run" => return fixture("ab.run"),
            "absent.qrels" | "absent.run" => return fixture(name),
            "malformed.qrels" => "1 0 a\n",
            "malformed.run" => "1 Q0 a 1\n",
            "empty.qrels" | "empty.run" => "",
            "repeat.qrels" => "1 0 a 1\n1 0 a 0\n",
            "repeat.run" => "1 Q0 a 1 2 r\n1 Q0 a 2 1 r\n",
            _ => panic!("no input {name}"),
        };
        write(name, text)
    }

    fn status(options: &[&str], qrels: &str, run: &str) -> Option<i32> {
        eval(options, &input(qrels), &input(run)).status.code()
    }

    /// Checks that te-rust exits with `status` and names `file` on stderr.
    fn assert_reported(options: &[&str], qrels: &Path, run: &Path, status: i32, file: &str) {
        let out = eval(options, qrels, run);
        assert_eq!(
            out.status.code(),
            Some(status),
            "{options:?}: {}",
            stderr(&out)
        );
        assert!(
            stderr(&out).contains(file),
            "{file} is not in {}",
            stderr(&out)
        );
    }

    #[test]
    fn unreadable_or_malformed_input_exits_2() {
        let map = &["-m", "map"][..];
        let jg = &["-R", "qrels_jg", "-m", "map_avgjg"][..];
        for (options, qrels, run) in [
            (map, "absent.qrels", "good.run"),
            (map, "good.qrels", "absent.run"),
            (map, "malformed.qrels", "good.run"),
            (map, "good.qrels", "malformed.run"),
            (map, "empty.qrels", "good.run"),
            (map, "good.qrels", "empty.run"),
            (&["-R", "nosuch"], "good.qrels", "good.run"),
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
        for measure in [
            "P.x",
            "map_cut.x",
            "rbp.p=0.2,p=0.8",
            "rbp_resid.p=0.2,p=0.8",
            "rbp_resid.x=1",
            "rbp_resid.1=2,01=3",
        ] {
            let out = eval(&["-m", measure], &input("good.qrels"), &input("good.run"));
            assert_eq!(out.status.code(), Some(2), "{measure}");
            assert!(out.stdout.is_empty(), "{measure}");
        }
    }

    #[test]
    fn an_illegal_measure_name_exits_1_before_the_files_are_read() {
        for (measures, qrels, run) in [
            (&["nosuch"][..], "good.qrels", "good.run"),
            (&["P."], "good.qrels", "good.run"),
            (&["nosuch"], "good.qrels", "absent.run"),
            (&["nosuch"], "malformed.qrels", "repeat.run"),
            // A name is checked before any parameter, whatever their order.
            (&["P.x", "nosuch"], "good.qrels", "good.run"),
            (&["P.x", "P."], "good.qrels", "good.run"),
        ] {
            let options: Vec<&str> = measures.iter().flat_map(|m| ["-m", m]).collect();
            assert_eq!(
                status(&options, qrels, run),
                Some(1),
                "{measures:?} {qrels} {run}"
            );
        }
    }

    #[test]
    fn a_repeated_document_in_a_scored_topic_exits_4() {
        assert_eq!(status(&["-m", "map"], "repeat.qrels", "good.run"), Some(4));
        assert_eq!(status(&["-m", "map"], "good.qrels", "repeat.run"), Some(4));
    }

    #[test]
    fn usage_errors_exit_1() {
        let out = te_rust()
            .args(["-m", "map"])
            .arg(input("good.qrels"))
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert_eq!(status(&["--nosuchflag"], "good.qrels", "good.run"), Some(1));
    }

    #[test]
    fn success_and_help_exit_0() {
        assert_eq!(status(&["-m", "map"], "good.qrels", "good.run"), Some(0));
        for flag in ["--help", "--version"] {
            assert_eq!(
                te_rust().arg(flag).output().unwrap().status.code(),
                Some(0),
                "{flag}"
            );
        }
    }

    #[test]
    fn the_qrels_are_checked_before_the_run() {
        let map = &["-m", "map"];
        assert_reported(
            map,
            &input("absent.qrels"),
            &input("absent.run"),
            2,
            "absent.qrels",
        );
        let qrels = input("malformed.qrels");
        assert_reported(
            map,
            &qrels,
            &input("repeat.run"),
            2,
            &qrels.to_string_lossy(),
        );
    }

    #[test]
    fn the_judgment_format_is_checked_before_the_run() {
        let options = &["-R", "nosuch"];
        assert_reported(
            options,
            &input("good.qrels"),
            &input("repeat.run"),
            2,
            "nosuch",
        );
    }

    #[test]
    fn a_repeated_judgment_is_found_after_the_run_is_read() {
        let (qrels, run) = (input("repeat.qrels"), input("malformed.run"));
        assert_reported(&["-m", "map"], &qrels, &run, 2, &run.to_string_lossy());
        assert_reported(
            &["-m", "map"],
            &qrels,
            &input("good.run"),
            4,
            &qrels.to_string_lossy(),
        );
    }

    #[test]
    fn an_unusable_parameter_exits_2_whatever_the_files() {
        // trec_eval reports the file and te-rust the parameter, with the same status.
        for (qrels, run) in [
            ("absent.qrels", "good.run"),
            ("good.qrels", "absent.run"),
            ("repeat.qrels", "good.run"),
        ] {
            assert_eq!(status(&["-m", "P.x"], qrels, run), Some(2), "{qrels} {run}");
        }
    }
}

/// Failed writes to standard output. trec_eval ignores them and exits 0; te-rust reports them.
#[cfg(unix)]
mod output {
    use super::common::{rows, stderr, te_rust, test_data, write};
    use std::fs::File;
    use std::os::unix::process::ExitStatusExt;
    use std::process::{Command, Output, Stdio};

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
        // The reader of the pipe has exited, so the first write fails.
        let mut reader = Command::new("true").stdin(Stdio::piped()).spawn().unwrap();
        let writer = reader.stdin.take().unwrap();
        reader.wait().unwrap();
        let output = te_rust()
            .args(["-q", "-m", "all_trec"])
            .arg(test_data("qrels.test"))
            .arg(test_data("results.test"))
            .stdout(writer)
            .output()
            .unwrap();
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
        let (qrels, run) = (test_data("qrels.test"), test_data("results.test"));
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
    fn failed_message_write_still_exits_8() {
        // With stderr on the same full disk, the message is lost but the status is not.
        let (qrels, run) = (test_data("qrels.test"), test_data("results.test"));
        let output = Command::new("sh")
            .arg("-c")
            .arg("trap '' XFSZ; ulimit -f 0; exec \"$0\" \"$@\" 2>&1")
            .arg(env!("CARGO_BIN_EXE_te-rust"))
            .arg(qrels)
            .arg(run)
            .stdout(file("both.out"))
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(EXIT_OUTPUT),
            "{:?}",
            output.status
        );
    }

    #[test]
    fn failed_error_message_write_keeps_the_status() {
        let qrels = test_data("qrels.test");
        for args in [
            &[qrels.to_str().unwrap(), "absent.run"][..],
            &["--help-measure", "nosuch"],
        ] {
            let status = |stderr: Stdio| {
                te_rust_with_no_room(args)
                    .stderr(stderr)
                    .output()
                    .unwrap()
                    .status
                    .code()
            };
            let expected = status(Stdio::piped());
            assert_ne!(expected, Some(0), "{args:?}");
            assert_eq!(status(file("error.out").into()), expected, "{args:?}");
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn full_device_exits_with_a_message() {
        let full = File::options().write(true).open("/dev/full").unwrap();
        let output = te_rust()
            .arg(test_data("qrels.test"))
            .arg(test_data("results.test"))
            .stdout(full)
            .output()
            .unwrap();
        assert_write_failure(&output);
    }

    #[test]
    fn successful_writes_print_every_row() {
        // qrels.test and results.test share topics 301, 302 and 303.
        let output = te_rust()
            .args(["-m", "num_q"])
            .arg(test_data("qrels.test"))
            .arg(test_data("results.test"))
            .output()
            .unwrap();
        assert_eq!(rows(&output), ["num_q all 3"]);
        assert!(output.stderr.is_empty(), "{}", stderr(&output));
    }
}

/// The differences from trec_eval that docs/cli.md lists. A failure here means that list needs
/// updating.
mod documented_differences {
    use super::common::{te_rust, test_data};
    use std::fs::File;
    use std::process::Output;

    /// Runs te-rust with results.test on standard input.
    fn te_rust_with_stdin(args: &[&str]) -> Output {
        let stdin = File::open(test_data("results.test")).unwrap();
        te_rust().args(args).stdin(stdin).output().unwrap()
    }

    fn eval(options: &[&str], run: &str) -> Output {
        let qrels = test_data("qrels.test");
        let args: Vec<&str> = options
            .iter()
            .copied()
            .chain([qrels.to_str().unwrap(), run])
            .collect();
        te_rust_with_stdin(&args)
    }

    #[test]
    fn listed_trec_eval_features_are_refused() {
        let run = test_data("results.test");
        let run = run.to_str().unwrap();
        for options in [
            &["-T", "trec_results"][..],
            &["-D", "1"],
            &["-Z", "zscores"],
            &["-o"],
            &["--Max_retrieved_per_topic=5"],
            &["--complete_rel_info_wanted"],
            &["-m", "prefs_simp"],
            &["-R", "prefs"],
            &["-R", "qrels_prefs"],
        ] {
            assert!(!eval(options, run).status.success(), "{options:?}");
        }
        // trec_eval reads the run from standard input when its name is "-".
        assert!(!eval(&["-m", "map"], "-").status.success());
        assert!(!te_rust_with_stdin(&["-v"]).status.success());
        assert!(te_rust_with_stdin(&["-V"]).status.success());
    }

    #[test]
    fn listed_extensions_are_accepted() {
        let run = test_data("results.test");
        for (measure, row) in [
            ("P_5", "P_5"),
            ("P_avgjg_5", "P_avgjg_5"),
            ("recall_10", "recall_10"),
            ("ndcg_cut_10", "ndcg_cut_10"),
            ("map_cut_100", "map_cut_100"),
            ("success_5", "success_5"),
            ("unj_10", "unj_10"),
            ("P@5", "P_5"),
            ("recall@10", "recall_10"),
            ("success@5", "success_5"),
            ("unj@10", "unj_10"),
            ("relative_P@5", "relative_P_5"),
            ("ndcg@10", "ndcg_cut_10"),
            ("map@100", "map_cut_100"),
            ("mrr", "recip_rank"),
            ("rr", "recip_rank"),
            ("precision.5", "P_5"),
            ("MAP", "map"),
            ("ndcg_p", "ndcg_p"),
        ] {
            let output = eval(&["-m", measure], run.to_str().unwrap());
            assert!(output.status.success(), "{measure}");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout
                    .lines()
                    .any(|line| line.split_whitespace().next() == Some(row)),
                "{measure} did not print {row}: {stdout}"
            );
        }
    }

    #[test]
    fn rel_format_help_lists_only_supported_formats() {
        let output = te_rust_with_stdin(&["--help"]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout
            .lines()
            .find(|line| line.contains("--rel-format"))
            .expect("no -R line in --help");
        assert!(line.contains("qrels_jg"), "{line}");
        assert!(!line.contains("prefs"), "{line}");
    }

    /// docs/cli.md lists `-l=2` as a te-rust spelling of `-l 2`.
    #[test]
    fn short_option_value_after_equals_is_read_like_a_separate_value() {
        let run = test_data("results.test");
        let run = run.to_str().unwrap();
        for (joined, separate) in [
            (&["-l=2", "-m", "map"][..], &["-l", "2", "-m", "map"][..]),
            (&["-M=1", "-m", "P.1,2"], &["-M", "1", "-m", "P.1,2"]),
            (
                &["-N=10", "-m", "utility.0,0,0,1"],
                &["-N", "10", "-m", "utility.0,0,0,1"],
            ),
            (&["-m=recip_rank"], &["-m", "recip_rank"]),
            (&["-R=qrels", "-m", "map"], &["-R", "qrels", "-m", "map"]),
        ] {
            let (joined_out, separate_out) = (eval(joined, run), eval(separate, run));
            assert!(separate_out.status.success(), "{separate:?}");
            assert!(!separate_out.stdout.is_empty(), "{separate:?}");
            assert_eq!(
                joined_out.status.code(),
                separate_out.status.code(),
                "{joined:?}"
            );
            assert_eq!(joined_out.stdout, separate_out.stdout, "{joined:?}");
        }
    }
}

/// Which measures te-rust evaluates for its `-m` options, and the order of their rows.
mod measure_requests {
    use super::common::{assert_refused, eval, fixture, rows};

    /// The measure and topic of each row for `options` on `name`.qrels and `name`.run.
    fn printed(name: &str, options: &[&str]) -> Vec<String> {
        let out = eval(
            options,
            &fixture(&format!("{name}.qrels")),
            &fixture(&format!("{name}.run")),
        );
        rows(&out)
            .iter()
            .map(|row| row.rsplit_once(' ').unwrap().0.to_string())
            .collect()
    }

    #[test]
    fn measures_print_in_trec_eval_table_order() {
        // trec_eval prints num_q before map, whatever the order of the -m options.
        assert_eq!(
            printed("topics", &["-m", "map", "-m", "num_q"]),
            ["num_q all", "map all"]
        );
        // trec_eval's output for the reversed list.
        assert_eq!(
            printed(
                "topics",
                &["-m", "unj.5", "-m", "rbp", "-m", "P.5", "-m", "map", "-m", "runid"]
            ),
            ["runid all", "map all", "P_5 all", "rbp all", "unj_5 all"]
        );
    }

    #[test]
    fn topic_rows_follow_the_same_order() {
        // trec_eval -q prints map before recip_rank in each topic and in the summary.
        assert_eq!(
            printed("topics", &["-q", "-m", "recip_rank", "-m", "map"]),
            [
                "map 1",
                "recip_rank 1",
                "map 2",
                "recip_rank 2",
                "map all",
                "recip_rank all"
            ]
        );
    }

    #[test]
    fn measures_that_trec_eval_lacks_print_last() {
        // trec_eval refuses ndcg_p, so te-rust prints it after the trec_eval measures.
        assert_eq!(
            printed("topics", &["-m", "ndcg_p", "-m", "map"]),
            ["map all", "ndcg_p all"]
        );
    }

    #[test]
    fn group_order_is_trec_eval_order() {
        assert_eq!(
            printed("topics", &["-m", "official"])[..7],
            [
                "runid all",
                "num_q all",
                "num_ret all",
                "num_rel all",
                "num_rel_ret all",
                "map all",
                "gm_map all"
            ]
        );
    }

    #[test]
    fn repeated_measure_prints_once() {
        // All six documents are relevant and retrieved, so the single row is 1.
        let (qrels, run) = (fixture("six-relevant.qrels"), fixture("six-relevant.run"));
        assert_eq!(
            rows(&eval(&["-m", "map", "-m", "map"], &qrels, &run)),
            ["map all 1.0000"]
        );
        assert_eq!(
            printed("six-relevant", &["-q", "-m", "map", "-m", "map"]),
            printed("six-relevant", &["-q", "-m", "map"])
        );
    }

    #[test]
    fn group_and_member_print_once() {
        let official = printed("six-relevant", &["-m", "official"]);
        assert_eq!(
            printed("six-relevant", &["-m", "official", "-m", "map"]),
            official
        );
        assert_eq!(
            printed("six-relevant", &["-m", "map", "-m", "official"]),
            official
        );
        let (official_precision, official_rest): (Vec<_>, Vec<_>) =
            official.iter().partition(|row| row.starts_with("P_"));
        assert!(!official_precision.is_empty());
        for options in [
            ["-m", "official", "-m", "P.5"],
            ["-m", "P.5", "-m", "official"],
        ] {
            let rows = printed("six-relevant", &options);
            let (precision, rest): (Vec<_>, Vec<_>) =
                rows.iter().partition(|row| row.starts_with("P_"));
            assert_eq!(precision, ["P_5 all"], "{options:?}");
            assert_eq!(rest, official_rest, "{options:?}");
        }
    }

    #[test]
    fn aliases_and_reordered_lists_print_once() {
        assert_eq!(
            printed("six-relevant", &["-m", "P.5", "-m", "P_5", "-m", "P@5"]),
            ["P_5 all"]
        );
        assert_eq!(
            printed("six-relevant", &["-m", "P.5,10", "-m", "P.10,5"]),
            ["P_5 all", "P_10 all"]
        );
    }

    #[test]
    fn first_explicit_parameter_list_wins() {
        // trec_eval reads only the first parameter list for a measure.
        for (first, second, expected) in [
            ("P.2", "P.3", &["P_2 all"][..]),
            ("P.2,5", "P.5,10", &["P_2 all", "P_5 all"]),
            ("P.2,5", "P.5", &["P_2 all", "P_5 all"]),
            ("P.2", "P.2,3", &["P_2 all"]),
            (
                "Rprec_mult.0.219",
                "Rprec_mult.0.221,1",
                &["Rprec_mult_0.22 all"],
            ),
            ("P", "P.2", &["P_2 all"]),
        ] {
            assert_eq!(
                printed("six-relevant", &["-m", first, "-m", second]),
                expected,
                "{first} {second}"
            );
        }
    }

    #[test]
    fn global_gains_follow_the_rules_of_a_gain_map() {
        let (qrels, run) = (fixture("topics.qrels"), fixture("topics.run"));
        let out = eval(&["--global-gains", "1=x", "-m", "ndcg"], &qrels, &run);
        assert_refused(&out, "invalid gain 'x' for relevance level 1");
        let out = eval(&["--global-gains", "1=2,1=5", "-m", "ndcg"], &qrels, &run);
        assert_refused(&out, "relevance level 1 is given more than once");
    }
}

/// The integer options -l, -M and -N and integer cutoffs, which trec_eval reads with
/// `str_to_long`: C-locale whitespace, a sign and digits, with a warning for what follows them.
mod integer_options {
    use super::common::{assert_refused, eval, rows, stderr, test_data, write};
    use std::process::Output;

    /// b (grade 2) is at rank 2 and a (grade 1) at rank 3.
    fn graded(options: &[&str]) -> Output {
        let qrels = write("graded.qrels", "1 0 a 1\n1 0 b 2\n1 0 c 0\n");
        let run = write("graded.run", "1 Q0 c 1 3 r\n1 Q0 b 2 2 r\n1 Q0 a 3 1 r\n");
        eval(options, &qrels, &run)
    }

    /// `options` scores like `integer`, and warns that `value` was read as `read`.
    fn assert_reads_as(options: &[&str], integer: &[&str], value: &str, read: &str) {
        let (partial, whole) = (graded(options), graded(integer));
        assert_eq!(rows(&partial), rows(&whole), "{options:?}");
        let warning = format!("value '{value}' partially read as '{read}'");
        assert!(
            stderr(&partial).contains(&warning),
            "{options:?}: {}",
            stderr(&partial)
        );
        assert!(whole.stderr.is_empty(), "{integer:?}");
    }

    #[test]
    fn cutoffs_read_their_leading_integer_with_a_warning() {
        // trec_eval: "Warning, value `5x' partially read as `5'." and P_5.
        assert_reads_as(&["-m", "P.5x"], &["-m", "P.5"], "5x", "5");
        assert_reads_as(&["-m", "P.1.5"], &["-m", "P.1"], "1.5", "1");
        assert_reads_as(&["-m", "map_cut.2,3x"], &["-m", "map_cut.2,3"], "3x", "3");
    }

    #[test]
    fn relevance_level_and_document_counts_read_their_leading_integer_with_a_warning() {
        // At level 2, map = 1/2; at level 1, map = (1/2 + 2/3) / 2.
        assert_reads_as(
            &["-l", "2.5", "-m", "map"],
            &["-l", "2", "-m", "map"],
            "2.5",
            "2",
        );
        assert_reads_as(
            &["-M", "2x", "-m", "map"],
            &["-M", "2", "-m", "map"],
            "2x",
            "2",
        );
        assert_reads_as(
            &["-N", "10x", "-m", "utility.0,0,0,1"],
            &["-N", "10", "-m", "utility.0,0,0,1"],
            "10x",
            "10",
        );
    }

    #[test]
    fn integer_options_skip_leading_c_locale_whitespace() {
        for (option, value, measure, row) in [
            ("-l", " 2", "map", "map all 0.5000"),
            ("-M", "\t2", "map", "map all 0.2500"),
            ("-N", " 10", "utility.0,0,0,1", "utility_0,0,0,1 all 7.0000"),
        ] {
            let out = graded(&[option, value, "-m", measure]);
            assert_eq!(rows(&out), [row], "{option}");
            assert!(out.stderr.is_empty(), "{option}: {}", stderr(&out));
        }
    }

    #[test]
    fn values_without_a_leading_integer_are_refused() {
        // trec_eval: "could not be parsed", and "out of range" for a value beyond a long.
        for (options, message) in [
            (&["-m", "P.x"][..], "invalid integer cutoff 'x'"),
            (&["-m", "P..5"], "invalid integer cutoff '.5'"),
            (&["-l", ".5", "-m", "map"], "value '.5' could not be parsed"),
            (&["-l", "x", "-m", "map"], "value 'x' could not be parsed"),
            (
                &["-l", "9223372036854775808", "-m", "map"],
                "value '9223372036854775808' out of range",
            ),
            (&["-M", "x", "-m", "map"], "value 'x' could not be parsed"),
            (&["-M-1", "-m", "map"], "value '-1' is negative"),
            (&["-N", "x", "-m", "map"], "value 'x' could not be parsed"),
        ] {
            assert_refused(&graded(options), message);
        }
    }

    #[test]
    fn makefile_meas_params_check_reads_23_4_as_23() {
        // The Makefile's quicktest runs `-mndcg_cut.10,20,23.4 -msuccess.2,5,20` and expects
        // test/out.test.meas_params, which has ndcg_cut_23.
        let options = ["-q", "-mndcg_cut.10,20,23.4", "-msuccess.2,5,20"];
        let out = eval(
            &options,
            &test_data("qrels.test"),
            &test_data("results.test"),
        );
        let expected = std::fs::read_to_string(test_data("out.test.meas_params")).unwrap();
        let cutoff_rows = |rows: Vec<String>| {
            let mut rows: Vec<_> = rows
                .into_iter()
                .filter(|row| row.starts_with("ndcg_cut_") || row.starts_with("success_"))
                .collect();
            rows.sort();
            rows
        };
        let expected = expected
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "));
        let actual = cutoff_rows(rows(&out));
        assert_eq!(actual.len(), 24);
        assert_eq!(actual, cutoff_rows(expected.collect()));
    }
}

/// Relevance levels that select no judged document, given with -l.
mod relevance_level {
    use super::common::{fixture, summary, te_rust};
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
    fn huge_level_finishes_with_nothing_relevant() {
        assert!(te_rust_within_limit(&["-l", "1000000000000"])
            .status
            .success());
        for measure in ["bpref", "gm_bpref", "map"] {
            let out = te_rust_within_limit(&["-l", "1000000000000", "-m", measure]);
            assert_eq!(summary(&out, measure), "0.0000", "{measure}");
        }
        let out = te_rust_within_limit(&["-l", "9223372036854775807", "-m", "map"]);
        assert_eq!(summary(&out, "map"), "0.0000");
    }

    #[test]
    fn negative_level_finishes() {
        let out = te_rust_within_limit(&["-l-1", "-m", "bpref", "-m", "gm_bpref", "-m", "num_q"]);
        assert_eq!(summary(&out, "num_q"), "2");
    }

    #[test]
    fn bpref_at_ordinary_levels() {
        // Topic 1 at -l 1: d1 and d3 relevant, one nonrelevant d2 above d3. bpref = (1 + 0) / 2.
        // Topic 2 at -l 1: d2 relevant and first. bpref = 1. Mean = 0.75.
        let out = te_rust_within_limit(&["-l", "1", "-m", "bpref"]);
        assert_eq!(summary(&out, "bpref"), "0.7500");
        // At -l 2, d3 alone is relevant in topic 1 and ranks below two nonrelevant documents.
        // Topic 2 has nothing relevant. Both score 0.
        let out = te_rust_within_limit(&["-l", "2", "-m", "bpref"]);
        assert_eq!(summary(&out, "bpref"), "0.0000");
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
    fn topics_shared_with_the_run_are_scored() {
        // Topic 1: (1 + 2/3) / 2 with d1 and d3 relevant at ranks 1 and 3. Topic 2: 1.
        let out = eval(
            &["-m", "map"],
            &fixture("topics.qrels"),
            &fixture("topics.run"),
        );
        assert_eq!(summary(&out, "map"), "0.9167");
    }

    #[test]
    fn no_shared_topic_exits_4_without_complete() {
        let (qrels, run) = (fixture("topics.qrels"), topic_3_run());
        let out = eval(&["-m", "map"], &qrels, &run);
        assert_eq!(out.status.code(), Some(4));
        assert_refused(&out, "no topic has both results and judgments");
        assert!(!eval(&["-R", "qrels_jg", "-m", "map_avgjg"], &qrels, &run)
            .status
            .success());
        assert_eq!(
            summary(&eval(&["-c", "-m", "map"], &qrels, &run), "map"),
            "0.0000"
        );
    }

    #[test]
    fn complete_set_prints_the_average_over_qrels_topics() {
        let options = ["-c", "-m", "map", "-m", "num_q", "-m", "gm_map"];
        let out = eval(&options, &fixture("topics.qrels"), &topic_3_run());
        assert_eq!(summary(&out, "num_q"), "2");
        assert_eq!(summary(&out, "map"), "0.0000");
        // The floor of 1e-5 rounds to 0.0000 at four decimals.
        assert_eq!(summary(&out, "gm_map"), "0.0000");
        let options = ["-R", "qrels_jg", "-c", "-m", "map_avgjg"];
        let out = eval(&options, &fixture("topics.qrels"), &topic_3_run());
        assert_eq!(summary(&out, "map_avgjg"), "0.0000");
    }

    #[test]
    fn confidence_interval_rows_print_without_shared_topics() {
        let options = ["-c", "-C", "--seed", "1", "-m", "map"];
        let out = eval(&options, &fixture("topics.qrels"), &topic_3_run());
        assert_eq!(
            rows(&out),
            [
                "map all 0.0000",
                "map_ci_lower all 0.0000",
                "map_ci_upper all 0.0000"
            ]
        );
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
