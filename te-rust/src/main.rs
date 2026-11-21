use clap::Parser;
use std::io::{self, Write};
use std::process;

use te_rust::eval::bootstrap::BootstrapConfig;
use te_rust::eval::{evaluate_run, evaluate_run_jg, DuplicateSource, EvaluationError, EvaluationOutput};
use te_rust::io::{parse_trec_qrels, parse_trec_qrels_jg, parse_trec_run, RunData};
use te_rust::metrics::registry::MeasureSpec;
use te_rust::metrics::{EvalConfig, Measure, MetricValue, ValueFormat};
use te_rust::metrics;

// std's eprintln! panics when stderr cannot be written, as on a full disk.
// This one shadows it in this file and drops the message, as trec_eval does.
macro_rules! eprintln {
    ($($arg:tt)*) => {{
        use std::io::Write as _;
        let _ = writeln!(std::io::stderr(), $($arg)*);
    }};
}



#[derive(Parser, Debug)]
#[command(
    name = "te-rust",
    version = "0.1.0",
    about = "Rust clean-room implementation of trec_eval"
)]
struct Args {
    /// In addition to summary evaluation, print evaluation for each query/topic
    #[arg(short = 'q')]
    query_flag: bool,

    /// Print no summary averages/totals at the end
    #[arg(short = 'n')]
    no_summary_flag: bool,

    /// Minimum relevance level at which a document is considered relevant
    #[arg(short = 'l', default_value = "1", value_parser = leading_integer, allow_hyphen_values = true)]
    relevance_level: i64,

    /// Average over the complete set of queries in qrels instead of the intersection
    #[arg(short = 'c')]
    complete_set_average: bool,

    /// Remove all unjudged documents from the retrieved set before evaluating
    #[arg(short = 'J')]
    judged_docs_only: bool,

    /// Max number of documents per topic to evaluate
    #[arg(short = 'M', value_parser = leading_count)]
    max_docs_per_topic: Option<usize>,

    /// Number of documents in the collection
    #[arg(short = 'N', default_value = "0", value_parser = leading_count)]
    num_docs_in_coll: usize,

    /// Calculate only the indicated measure(s)
    #[arg(short = 'm', value_name = "measure")]
    measures: Vec<String>,

    /// Compute and print bootstrap confidence intervals for summary averages
    #[arg(short = 'C', long = "ci")]
    ci: bool,

    /// Format confidence intervals in bracketed human-readable format [lower, upper]
    #[arg(long = "ci-pretty")]
    ci_pretty: bool,

    /// Significance level alpha for confidence intervals (default: 0.05 for 95% CI)
    #[arg(long = "ci-alpha", default_value = "0.05")]
    ci_alpha: f64,

    /// Number of bootstrap resamples (default: 1000)
    #[arg(long = "ci-samples", default_value = "1000")]
    ci_samples: usize,

    /// Optional RNG seed for deterministic bootstrap resampling
    #[arg(long = "seed")]
    seed: Option<u64>,

    /// List all available measures with a short description and exit
    #[arg(long = "help-measures")]
    help_measures: bool,

    /// Print detailed explanation of the specified measure and exit
    #[arg(long = "help-measure", value_name = "name")]
    help_measure: Option<String>,

    /// Format of the relevance judgments file: qrels or qrels_jg
    #[arg(short = 'R', long = "rel-format", default_value = "qrels")]
    rel_format: String,

    /// Path to relevance judgments (qrels) file
    qrels_file: Option<String>,


    /// Path to run results file
    run_file: Option<String>,

    /// Global relevance-to-gain mapping (e.g. --global-gains 1=3.5,2=9.0)
    #[arg(long = "global-gains", value_name = "gains", value_parser = metrics::common::GainsConfig::parse)]
    global_gains: Option<metrics::common::GainsConfig>,
}

// CLI exit statuses: 1 usage or illegal measure name, 2 input or measure
// parameter error, 4 topic evaluation error, 8 output write error.
// Status 8 is an extension; trec_eval ignores output write errors.
const EXIT_USAGE: i32 = 1;
const EXIT_INPUT: i32 = 2;
const EXIT_TOPIC: i32 = 4;
const EXIT_OUTPUT: i32 = 8;

/// Reports an evaluation error with trec_eval's exit status.
fn refuse_evaluation_error(
    error: EvaluationError,
    qrels_kind: &str,
    qrels_file: &str,
    run_file: &str,
) -> ! {
    let EvaluationError::DuplicateDocument { source, .. } = error else {
        eprintln!("te-rust: {}", error);
        process::exit(EXIT_INPUT);
    };
    let (kind, file) = match source {
        DuplicateSource::Qrels => (qrels_kind, qrels_file),
        DuplicateSource::Run => ("run", run_file),
    };
    eprintln!("Error in {} file '{}': {}", kind, file, error);
    process::exit(EXIT_TOPIC);
}

/// Reads an integer option as trec_eval's `str_to_long` does: the leading integer, so `1.5` reads
/// as 1, with a warning when characters follow it.
fn leading_integer(value: &str) -> Result<i64, String> {
    match te_rust::io::parsers::read_leading_integer(value) {
        Ok((integer, partial)) => {
            if partial {
                eprintln!("te-rust: Warning, value '{}' partially read as '{}'", value, integer);
            }
            Ok(integer)
        }
        Err(std::num::IntErrorKind::PosOverflow | std::num::IntErrorKind::NegOverflow) => {
            Err(format!("value '{}' out of range", value))
        }
        Err(_) => Err(format!("value '{}' could not be parsed", value)),
    }
}

/// As [`leading_integer`], for a count that cannot be negative.
fn leading_count(value: &str) -> Result<usize, String> {
    usize::try_from(leading_integer(value)?).map_err(|_| format!("value '{}' is negative", value))
}

/// Reads the run file and prints its warnings, or exits with EXIT_INPUT when it cannot.
fn read_run(path: &str) -> RunData {
    match parse_trec_run(path) {
        Ok(data) => {
            for warning in &data.warnings {
                eprintln!("Warning in run file '{}': {}", path, warning);
            }
            data
        }
        Err(e) => {
            eprintln!("Error parsing run file '{}': {}", path, e);
            process::exit(EXIT_INPUT);
        }
    }
}

fn handle_help_flags(help_measures: bool, help_measure: Option<&str>) {
    if help_measures {
        check_write(print_measure_list(&mut io::stdout().lock()));
        process::exit(0);
    }

    if let Some(target) = help_measure {
        for spec in metrics::registry::registry() {
            if spec.name.eq_ignore_ascii_case(target) {
                if let Ok(m) = (spec.factory)("", &mut Vec::new()) {
                    check_write(print_measure_help(&mut io::stdout().lock(), spec, m.as_ref()));
                    process::exit(0);
                }
            }
        }
        eprintln!("te-rust: Unknown measure '{}'", target);
        process::exit(EXIT_USAGE);
    }
}

fn print_measure_list(out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "{:<22} {:<13} Description", "Measure", "Status")?;
    writeln!(out, "{}", "-".repeat(70))?;
    for spec in metrics::registry::registry() {
        if let Ok(m) = (spec.factory)("", &mut Vec::new()) {
            writeln!(out, "{:<22} {:<13} {}", m.name(), spec.status.label(), m.short_description())?;
        }
    }
    writeln!(out, "\nRequest measures with -m <name> (repeatable). See --help-measure <name> for details.")?;
    out.flush()
}

fn print_measure_help(out: &mut impl Write, spec: &MeasureSpec, m: &dyn Measure) -> io::Result<()> {
    writeln!(out, "{}", m.name())?;
    if !spec.status.label().is_empty() {
        writeln!(out, "Status: {}", spec.status.label())?;
    }
    writeln!(out)?;
    writeln!(out, "{}", m.explanation())?;
    writeln!(out)?;
    // How to request it, and what parameters it accepts.
    if spec.usage.is_empty() {
        writeln!(out, "Usage:  -m {}", m.name())?;
    } else {
        writeln!(out, "Usage:  -m {}", spec.usage)?;
    }
    out.flush()
}

// Rust ignores SIGPIPE. Restoring the default lets a closed pipe, as in
// `te-rust ... | head`, end te-rust as it ends trec_eval.
#[cfg(unix)]
fn restore_default_sigpipe() {
    // SAFETY: no other thread exists yet, and SIG_DFL is a valid disposition.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn restore_default_sigpipe() {}

fn main() {
    restore_default_sigpipe();

    let args = Args::try_parse().unwrap_or_else(|e| {
        let status = if e.use_stderr() { EXIT_USAGE } else { 0 };
        let result = e.print();
        if !e.use_stderr() {
            check_write(result);
        }
        process::exit(status);
    });

    // 1. Handle help flags if present
    if args.help_measures || args.help_measure.is_some() {
        handle_help_flags(args.help_measures, args.help_measure.as_deref());
    }

    // 2. Validate positional files
    let qrels_filename = match args.qrels_file {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:\n  <QRELS_FILE>\n  <RUN_FILE>\n\nUsage: te-rust [OPTIONS] <QRELS_FILE> <RUN_FILE>");
            process::exit(EXIT_USAGE);
        }
    };

    let run_filename = match args.run_file {
        Some(f) => f,
        None => {
            eprintln!("error: the following required arguments were not provided:\n  <RUN_FILE>\n\nUsage: te-rust [OPTIONS] <QRELS_FILE> <RUN_FILE>");
            process::exit(EXIT_USAGE);
        }
    };

    // 3. Resolve measures (groups + parameters) via the central registry.
    let default_group = match args.rel_format.to_ascii_lowercase().as_str() {
        "qrels_jg" => "qrels_jg",
        _ => "official",
    };

    let requested_names: Vec<String> = if args.measures.is_empty() {
        vec![default_group.to_string()]
    } else {
        args.measures.clone()
    };

    let resolved = match metrics::registry::resolve_measures(&requested_names) {
        Ok(resolved) => resolved,
        Err(e) => {
            eprintln!("te-rust: {}", e);
            process::exit(if e.illegal_name { EXIT_USAGE } else { EXIT_INPUT });
        }
    };
    for warning in &resolved.warnings {
        eprintln!("te-rust: Warning, {}", warning);
    }
    let active_measures = metrics::registry::in_trec_eval_order(resolved.measures);

    // 4. Setup runtime config
    let global_gains = args.global_gains;
    let config = EvalConfig {
        query_flag: args.query_flag,
        all_query_values: false,
        summary_flag: !args.no_summary_flag,
        relevance_level: args.relevance_level,
        average_complete_flag: args.complete_set_average,
        judged_docs_only_flag: args.judged_docs_only,
        max_num_docs_per_topic: args.max_docs_per_topic.unwrap_or(usize::MAX),
        num_docs_in_coll: args.num_docs_in_coll,
        global_gains,
    };
    if let Err(e) = config.validate() {
        eprintln!("te-rust: {}", e);
        process::exit(EXIT_USAGE);
    }

    let enable_ci = args.ci || args.ci_pretty;
    let ci_config = if enable_ci {
        Some(BootstrapConfig {
            num_samples: args.ci_samples,
            alpha: args.ci_alpha,
            seed: args.seed,
        })
    } else {
        None
    };

    // 5. Read the qrels, then the run, as trec_eval does, and evaluate
    let output = match args.rel_format.to_ascii_lowercase().as_str() {
        "qrels_jg" => {
            let qrels_jg_data = match parse_trec_qrels_jg(&qrels_filename) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error parsing qrels_jg file '{}': {}", qrels_filename, e);
                    process::exit(EXIT_INPUT);
                }
            };
            for warning in &qrels_jg_data.warnings {
                eprintln!("Warning in qrels_jg file '{}': {}", qrels_filename, warning);
            }
            let run_data = read_run(&run_filename);
            evaluate_run_jg(
                &qrels_jg_data,
                &run_data,
                &active_measures,
                &config,
                ci_config.as_ref(),
            )
            .unwrap_or_else(|error| {
                refuse_evaluation_error(error, "qrels_jg", &qrels_filename, &run_filename);
            })
        }
        "qrels" => {
            let qrels_data = match parse_trec_qrels(&qrels_filename) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error parsing qrels file '{}': {}", qrels_filename, e);
                    process::exit(EXIT_INPUT);
                }
            };
            for warning in &qrels_data.warnings {
                eprintln!("Warning in qrels file '{}': {}", qrels_filename, warning);
            }
            let run_data = read_run(&run_filename);
            evaluate_run(
                &qrels_data,
                &run_data,
                &active_measures,
                &config,
                ci_config.as_ref(),
            )
            .unwrap_or_else(|error| {
                refuse_evaluation_error(error, "qrels", &qrels_filename, &run_filename);
            })
        }
        other => {
            eprintln!("te-rust: Unsupported relevance judgments format '{}'", other);
            process::exit(EXIT_INPUT);
        }
    };

    if !config.average_complete_flag && output.num_queries_evaluated == 0 {
        eprintln!("te-rust: no topic has both results and judgments; -c evaluates each qrels topic missing from the run as an empty ranking");
        process::exit(EXIT_TOPIC);
    }

    // 6. Print the query results, then the summary
    check_write(print_results(&mut io::stdout().lock(), &output, &config, args.ci_pretty));
}

fn print_results(
    out: &mut impl Write,
    output: &EvaluationOutput,
    config: &EvalConfig,
    ci_pretty: bool,
) -> io::Result<()> {
    if config.query_flag {
        for q_eval in &output.query_results {
            for (name, val, format) in &q_eval.scores {
                print_metric_value(out, name, &q_eval.qid, val, *format)?;
            }
        }
    }

    if config.summary_flag {
        for s_eval in &output.summary_results {
            if let (Some(ci), true) = (&s_eval.ci, ci_pretty) {
                if let MetricValue::Float(f) = &s_eval.value {
                    writeln!(out, "{:<22}\tall\t{:.4} [{:.4}, {:.4}]", s_eval.name, f, ci.lower, ci.upper)?;
                }
            } else {
                print_metric_value(out, &s_eval.name, "all", &s_eval.value, s_eval.format)?;
                if let Some(ci) = &s_eval.ci {
                    let lower_name = format!("{}_ci_lower", s_eval.name);
                    let upper_name = format!("{}_ci_upper", s_eval.name);
                    print_metric_value(out, &lower_name, "all", &MetricValue::Float(ci.lower), ValueFormat::Float)?;
                    print_metric_value(out, &upper_name, "all", &MetricValue::Float(ci.upper), ValueFormat::Float)?;
                }
            }
        }
    }
    out.flush()
}

fn print_metric_value(out: &mut impl Write, name: &str, qid: &str, val: &MetricValue, format: ValueFormat) -> io::Result<()> {
    match (val, format) {
        (MetricValue::Float(f), ValueFormat::Float) => {
            writeln!(out, "{:<22}\t{}\t{:.4}", name, qid, f)
        }
        (MetricValue::Integer(i), ValueFormat::Integer) => {
            writeln!(out, "{:<22}\t{}\t{}", name, qid, i)
        }
        (MetricValue::Str(s), ValueFormat::Str) => {
            writeln!(out, "{:<22}\t{}\t{}", name, qid, s)
        }
        (MetricValue::Str(s), ValueFormat::QuotedStr) => {
            writeln!(out, "{:<22}\t{}\t'{}'", name, qid, s)
        }
        _ => {
            // Fallback
            match val {
                MetricValue::Float(f) => writeln!(out, "{:<22}\t{}\t{:.4}", name, qid, f),
                MetricValue::Integer(i) => writeln!(out, "{:<22}\t{}\t{}", name, qid, i),
                MetricValue::Str(s) => writeln!(out, "{:<22}\t{}\t{}", name, qid, s),
            }
        }
    }
}

/// Exits with EXIT_OUTPUT when writing to stdout failed, as on a full disk, where
/// println! would panic. On unix a closed pipe ends the process with SIGPIPE first.
fn check_write(result: io::Result<()>) {
    if let Err(e) = result {
        // A reader that stops early, as `head` does, needs no message.
        if e.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("te-rust: cannot write to standard output: {}", e);
        }
        process::exit(EXIT_OUTPUT);
    }
}
