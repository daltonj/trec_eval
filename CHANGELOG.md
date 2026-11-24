# Changelog

## Unreleased

Fixes for issues #57 to #60, #66, #67, #80, #162, #167, #173, #177 and #199 on usnistgov/trec_eval, and for defects found while fixing them (see [Also fixed](#also-fixed)); #173 changes only the regression tests. [docs/cli.md](docs/cli.md#differences-from-trec_eval) lists the differences from `trec_eval` that remain.

### Results that change

- Cutoff lists are sorted, as `trec_eval` sorts them. `map_cut`, `ndcg_cut`, `relative_P`, `unj`, `iprec_at_recall`, `Rprec_mult`, `Rprec_mult_avgjg` and `11pt_avg` give `trec_eval`'s values for cutoffs out of order, such as `-m map_cut.5,2`, and `-m 11pt_avg.1.0,0.5` gives the value of `11pt_avg.0.5,1.0` under its own row name. Every measure with a row per cutoff prints its rows in ascending order (#57).
- Under `-c`, `gm_map` and `gm_bpref` count each topic missing from the run once, not twice (#58).
- `rbp` no longer falls as `-l` rises above a topic's highest grade (#59).
- `rbp` applies its own gain map, such as `rbp.1=3,2=9`, which it ignored. When it rescales gains that fall outside [0, 1], it starts the range at 0, as `trec_eval` does, and an unjudged document keeps gain 0 unless the map gives level -1 a gain (#60 items 1 and 2).
- `ndcg_rel` gives `trec_eval`'s value with negative gains or a gain on unjudged documents. It is 0 when its corrected sum is not positive, and it applies the correction for relevant documents not retrieved whenever the ideal DCG is positive (#60 items 3 and 4). As in `trec_eval`, it can still exceed 1 when the map gives unjudged documents, level -1, a positive gain.
- Through Python and the Rust library, `Rprec_mult` and `Rprec_mult_avgjg` with a cutoff beyond `i64::MAX` divide by that cutoff: `Rprec_mult.1e19` gives 1e-19 for two relevant documents retrieved, not 2.17e-19. The CLI prints 0.0000 in both cases (#162 item 14).
- `relstring` with a limit near `u64::MAX` prints the grades of the documents retrieved, where te-rust panicked (#162 item 13).
- A run that shares no topic with the qrels exits with status 4 and a message, where te-rust printed nothing and exited 0. With `-c` it prints the average over the qrels topics (#80).
- Each measure prints once, and measures print in `trec_eval`'s order whatever the order of `-m`. Measures that `trec_eval` lacks, such as `ndcg_p`, follow in request order (#177 items 3 and 4). A measure requested twice, or as both `mrr` and `recip_rank`, prints one set of rows. As in `trec_eval`, a measure uses the first parameter list given for it, which replaces its default list or a group's: `-m official -m P.5` prints only `P_5`, and `-m ndcg -m ndcg.1=2` prints only `ndcg_1=2`. A later, different list is ignored with a warning.
- `dcg` and `ideal_dcg` are added, each with its own gain map (#177 item 2).

### Input

- Refused with exit status 2 and a message that names the request (#162):
  - Gain maps for `G`, `ndcg`, `ndcg_rel`, `Rndcg`, `ndcg_p`, `rbp` and `rbp_resid`: an entry that is not `<level>=<gain>`, a level that is not an integer or is given twice, or a gain that is not a finite number. te-rust skipped such an entry or kept the last gain; an `rbp` or `rbp_resid` entry without `=` was refused with exit status 1. `p` given twice for `rbp` or `rbp_resid` is refused too (items 1 and 2). The new `dcg` and `ideal_dcg` read gain maps the same way.
  - Cutoffs: an integer rank cutoff or a multiple of R must be positive, and a cutoff of a measure that prints a row per cutoff must not repeat. A recall level of 0 is accepted, `11pt_avg` recall levels may repeat, and `utility` takes four finite coefficients, which may be 0 or repeat (items 4 and 7). An integer cutoff above `i64::MAX` is refused (item 6), as are two floating-point cutoffs that print the same row name, such as `Rprec_mult.0.999,1.001` (item 9).
  - Values: a NaN, infinite or overflowing floating-point parameter (item 5); a recall level outside [0, 1], a negative `set_F` weight, and an `rbp` or `rbp_resid` persistence outside [0, 1) (items 7 and 8).
- Refused with exit status 1: `-m P.`, `-m P_` and `-m P@`, which printed the default cutoffs (#162 item 3), and `--global-gains` with an unusable gain map, which te-rust ignored (#162 item 1).
- Refused with exit status 2 and a message that names the line: a relevance level outside -127 to 127 in a qrels or qrels_jg file, where te-rust accepted -1,000,000 to 1,000,000 (#167 item 6); a NUL byte; a CR that ends neither a line nor the file; and a data line that is not UTF-8, which gave an I/O error without a line number (#167 item 4). A comment line need not be UTF-8.
- Read as `trec_eval` reads them (#177): a run score as C `strtod` reads it, so hexadecimal values, `inf`, `-inf` and `1e400` are read, characters after the number give a warning, and a value beyond the range of an `f64` is kept as ±inf or ±0 with a warning; NaN and a score with no leading number are still refused (item 7). A grade such as `1.5` or `0.9` is read as its leading integer with a warning (item 9). An integer cutoff, `-l`, `-M` or `-N` with trailing characters, such as `P.5x`, `ndcg_cut.23.4` or `-l 1.5`, is read with a warning, and a cutoff that starts with a non-ASCII space, which te-rust trimmed, is refused (item 10).
- A UTF-8 byte-order mark at the start of a file is skipped, and only ASCII white space separates fields: a non-breaking space stays inside a docno, and U+0085 no longer splits a field (#167 items 3 and 5).
- A blank line in a qrels or qrels_jg file is skipped with a warning (#177 item 11).
- A repeated document is refused only where `trec_eval` refuses it: when a requested measure reads the ranking, evaluation checks every judgment of a scored topic and the first `-M` documents of its ranking. `-m num_q`, a repeat cut off by `-M`, and a repeat in a topic that is not scored are accepted. The exit status is 4, not 1 (#177 item 8). A document repeated in one judgment group of a qrels_jg file is still refused as the file is read, now with status 2.
- A negative `-l` is refused with exit status 1 and `Unusable relevance level`, whether given as `-l -1`, `-l-1` or `-l=-1`. `-l-1` and `-l=-1` made `bpref` and `gm_bpref` hang and `utility`, `11pt_avg` and `Rprec_mult_avgjg` give wrong values (#66). A large `-l` no longer makes `bpref` and `gm_bpref` hang. A negative `-M` or `-N` is refused with exit status 1, not 2, and so is a value above 9223372036854775807, which te-rust accepted (#177 item 5).

### Command line

- Exit statuses follow `trec_eval`: 1 for a usage error, an unknown measure or an empty parameter list; 2 for an unreadable or malformed file, an unsupported `-R` format or an unusable measure parameter; 4 for an evaluation error, such as a repeated document or a run that shares no topic with the qrels. te-rust exited 1 for every failure, and 2 for an argument that its option parser refused (#177 item 5).
- With two problems, te-rust checks the relevance level, then every measure name, then the measure parameters, then reads the qrels and then the run: an unknown measure is reported before a bad parameter, and an unreadable qrels file before an unreadable run (#177 item 6). `trec_eval` reads the options in the order given and checks measure parameters after it reads the files, so it can report another problem first.
- A failed write to standard output exits with status 8 and a message, where te-rust panicked (exit 134). On a closed pipe, te-rust ends through SIGPIPE (exit 141), as `trec_eval` does. A failed write to standard error no longer panics (#67).
- Warnings go to standard error. A warning about an input file names the file and the line, and is printed once the whole file has been read. A warning about an option or a measure parameter starts with `te-rust: Warning,` (#177).
- A repeated document is reported as `Error in <kind> file '<path>': Line <n>: duplicate document '<docno>' for query '<qid>', first at line <m>`. For a file that is not a regular file, such as a FIFO, te-rust does not read the file again, and the message has no line numbers (#177 item 8). Before, the file reader refused a repeated document with its line number and exit status 1.
- Parse messages say `No data lines` for a file of comments or blank lines, not `File is empty`, and `expected 4 fields` for a qrels line, not `expected at least 4 fields` (#177 item 12).
- Help text: `-R` offers only `qrels` and `qrels_jg`, the formats that te-rust reads (#177 item 13). `--help-measure set_F` calls the parameter x, the beta^2 of F_beta (#162 item 12), and `--help-measure rbp` and `--help-measure rbp_resid` list the gain map (#60 item 1, #162 item 2). The measure pages under `docs/measures/` correct the definitions and groups that differed from the code, such as those of `Rprec_mult`, `ndcg_rel`, `Rndcg`, `utility` and `set_F` (#162 item 12, #177 item 14).

### Rust library

These changes break source compatibility:

- `evaluate_run`, `evaluate_run_jg`, `align_query` and `align_query_jg` return `Result<_, EvaluationError>`. `EvaluationError` reports a negative relevance level (`NegativeRelevanceLevel`), a repeated document (`DuplicateDocument`, with a `DuplicateSource`), a relevance level outside -127 to 127 (`RelevanceOutOfBounds`), a NaN score (`NanScore`), and, without `average_complete_flag`, a run that shares no topic with the qrels (`NoSharedTopic`). `align_query` panicked on a NaN score in a long ranking and on a grade near `i64::MAX` (#66, #80, #167 item 2, #177 items 1 and 8).
- The measure constructors of `P`, `recall`, `success`, `map_cut`, `ndcg_cut`, `relative_P`, `unj`, `P_avgjg`, `iprec_at_recall`, `Rprec_mult`, `Rprec_mult_avgjg`, `11pt_avg`, `utility`, `set_F`, `G`, `ndcg`, `ndcg_rel`, `Rndcg`, `ndcg_p`, `rbp` and `rbp_resid` return `Result<Self, MeasureParseError>`. The cutoff constructors sort their cutoffs and refuse the lists that the CLI refuses: an empty list, a rank cutoff or multiple of R that is not positive, and a repeated cutoff of a measure that prints a row per cutoff. `iprec_at_recall` accepts a recall level of 0, and `11pt_avg` accepts repeated levels (#57, #162 items 10 and 11). `RbpMeasure::new` and `RbpResidMeasure::new` take only the parameter string and read `p` from it. `GainsConfig::parse` returns a `Result`.
- `parse_int_cutoffs` takes a third argument, `warnings: &mut Vec<String>`, to which it adds a warning for a cutoff read in part.
- `MeasureSpec::factory` has the type `fn(&str, &mut Vec<String>) -> Result<Box<dyn Measure>, MeasureParseError>`, so a custom `MeasureSpec` needs a factory that takes the warnings vector.
- `MeasureParseError` has a public field `illegal_name`, which a struct literal or an exhaustive pattern must name. `MeasureParseError::new` sets it to false, and `MeasureParseError::illegal_name` sets it to true.
- `resolve_measures` returns `ResolvedMeasures`, which holds the measures and the warnings, and builds each measure once from the first parameter list given for it. `in_trec_eval_order` orders measures as `trec_eval` prints them.
- `QrelsData`, `QrelsJGData` and `RunData` have a `warnings` field. `EvalConfig` has an `all_query_values` field and a `validate` method.
- `ParseError` has the variants `NoDataLines`, `NulCharacter`, `CarriageReturn` and `InvalidUtf8`, and the `expected` field of `InvalidFieldCount` is a `FieldCount`.
- `parse_trec_qrels` and `parse_trec_run` keep a repeated document, where they returned `ParseError::DuplicateDocument`; evaluation refuses it (#177 item 8).

Other changes:

- `Measure::calc` of `utility`, `11pt_avg` and `Rprec_mult_avgjg` counts only judged documents as relevant retrieved, so a direct call at a negative relevance level gives `trec_eval`'s value instead of wrapping (#66).
- Through the library and Python, `utility` uses a collection size above `i64::MAX` as given, where its count of documents not retrieved wrapped to a negative number (#66).
- `QueryEvalState.rel_levels` holds at most 128 levels, where a large relevance level padded it to that level (#66).
- Added: `Measure::reads_ranking`, which defaults to `true` and is `false` for `num_q` and `runid`; `DcgMeasure`; `check_rel_level`, `REL_LEVELS` and `read_leading_integer`; `repeated_document_lines`; the re-export `te_rust::io::FieldCount`; and `Default` for `GainsConfig`.

### Python

- `Evaluator` and `evaluate` raise `ValueError` for a negative `relevance_level`, which could hang (#66); for an unusable measure parameter or `global_gains`, which were accepted or ignored (#162); for a grade outside -127 to 127 in any qrels form, given as an `int` or another integer type such as a NumPy integer, beyond 64 bits included, where a large grade aborted the interpreter and others raised `TypeError` or `OverflowError` or were scored (#167 item 1); for a document repeated where the CLI refuses it, where a mapping or iterable kept the last judgment or counted the document twice; and for a NaN score, which was scored or aborted the interpreter (#177 item 1).
- When a document repeated in a qrels or run file is refused, the `ValueError` is `ValueError("Error in <kind> file '<path>': Line <n>: duplicate document '<docno>' for query '<qid>', first at line <m>")`, as the CLI reports it. The line numbers are left out unless the file is a regular file and reading it again finds both lines. For in-memory input the message is `qrels: duplicate document ...` or `run: duplicate document ...` (#177 item 8).
- `evaluate` and `evaluate_arrays` raise `NoSharedTopicError`, a subclass of `ValueError` exported from `trec_eval`, when the run shares no topic with the qrels and `complete_set_average` is false. With `complete_set_average`, the aggregate now holds the average over the qrels topics; it was empty (#80).
- Files are read as the CLI reads them (#167, #177). Warnings about files and measure parameters are issued as `UserWarning`: at most ten for the measures and ten for each file, each followed by one that counts the rest.
- `Evaluator` takes `all_query_values`, which adds the per-topic values of `gm_map`, `gm_bpref`, `num_q` and `runid`.
- The pytrec_eval wrapper keeps unjudged documents by default (`judged_docs_only_flag=False`), returns every value as a float, with 0.0 for `runid` and `relstring`, returns per-topic `gm_map`, `gm_bpref`, `num_q` and `runid`, and leaves out a topic whose judgments are an empty mapping, as pytrec_eval does. Its `supported_measures` lists `infAP`, which it spelled `infap`, and adds `num_q`, `set_P`, `set_recall`, `dcg` and `ideal_dcg`. It merges the parameter lists of repeated requests written as pytrec_eval writes them, such as `P_5` and `P.10`, for the measures that print a row per value, so that te-rust's first-list rule keeps every requested row (#199). Other repeated requests, such as `P@5` and `P@10` or `P_avgjg.5` and `P_avgjg.10`, follow the first-list rule: te-rust keeps the first and warns about the rest. [docs/python/compat.md](docs/python/compat.md) describes the effect on results.

### Also fixed

These defects in te-python have no issue on usnistgov/trec_eval.

- A file name given to the Python API is encoded by `os.fsencode`, as `open` encodes it: a name that is not UTF-8 opens the file it names, and a name that cannot be encoded raises `UnicodeEncodeError`, where te-python replaced the unencodable characters and could open another file. A directory raises `IsADirectoryError`, where it was reported as not found, a file that cannot be read raises the `OSError` that `open` raises, such as `PermissionError`, where it raised `ValueError`, and another path that is not a regular file, such as a FIFO, is read.
- `evaluate_arrays`, and qrels or runs given as DataFrame columns, raise `ValueError` when the three inputs have different lengths, where te-python stopped at the shortest, and pass on an error raised by an input's iterator.
- A measure given to `Evaluator` that is neither a name nor a custom measure, an object with `calc_query` or a callable, raises `TypeError` when the `Evaluator` is made, where `measures=[None]` failed only at evaluation.
- `compare_against_baseline` without `measures` compares every measure that the baseline reports, where it raised `AttributeError`.
