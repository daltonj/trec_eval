# Changelog

## Unreleased

Fixes for issues #57 to #60, #66, #67, #80, #162, #167, #173, #177 and #199 on usnistgov/trec_eval. [docs/cli.md](docs/cli.md#differences-from-trec_eval) lists the differences from `trec_eval` that remain.

### Results that change

- Cutoff lists are sorted, as `trec_eval` sorts them: `map_cut`, `ndcg_cut`, `relative_P`, `unj`, `iprec_at_recall`, `Rprec_mult` and `Rprec_mult_avgjg` give `trec_eval`'s values for cutoffs out of order, and every measure prints its cutoffs in ascending order (#57).
- Under `-c`, `gm_map` and `gm_bpref` count each topic missing from the run once (#58).
- `rbp` does not change with `-l` (#59). It applies its gain map, and gives an unjudged document a gain of 0 when it rescales the gains (#60).
- `ndcg_rel` stays between 0 and 1 with negative gains or a gain on unjudged documents (#60).
- A run that shares no topic with the qrels exits with status 4 instead of printing nothing; with `-c` it prints the average over the qrels topics (#80).
- Each measure prints once, and measures print in `trec_eval`'s order, whatever the order of `-m` (#177).
- `dcg` and `ideal_dcg` are added (#177).

### Input

- Refused with exit status 2 and a message that names the parameter: measure parameters that te-rust cannot use, such as a gain map it cannot parse, a cutoff of 0 or one given twice, a NaN or infinite value, a recall level outside [0, 1], or two cutoffs that print the same row name (#162).
- Refused with exit status 2 and a message that names the line: a relevance level outside -127 to 127, a NUL byte, a CR that does not end a line, and a data line that is not UTF-8 (#167).
- Read as `trec_eval` reads them: run scores, as C `strtod` reads them, with a warning for a partial read or an out-of-range value; a grade such as `1.5`, read as 1 with a warning; and an integer option or cutoff with trailing characters, read with a warning (#177).
- A UTF-8 byte-order mark at the start of a file is skipped, and only ASCII white space separates fields (#167). A blank line in a qrels file is skipped with a warning (#177).
- A repeated document is refused only where `trec_eval` refuses it: in a scored topic, among the first `-M` documents, and when a measure reads the ranking. The exit status is 4 (#177).
- A negative `-l` is refused with exit status 1, and a large `-l` no longer makes te-rust hang (#66).

### Command line

- Exit statuses follow `trec_eval`: 1 for a usage error or an unknown measure, 2 for unusable input or parameters, 4 for an evaluation error. A failed write to standard output exits with status 8 instead of a panic; on a closed pipe, te-rust ends through SIGPIPE as `trec_eval` does (#67, #177).
- Warnings go to standard error and name the file and the line. A repeated document is reported with the lines that give it.

### Rust library

These changes break source compatibility:

- `evaluate_run`, `evaluate_run_jg`, `align_query` and `align_query_jg` return `Result<_, EvaluationError>`. `EvaluationError` reports a repeated document (`DuplicateDocument`, with a `DuplicateSource`), a relevance level outside -127 to 127, a negative relevance level, a NaN score, and a run that shares no topic with the qrels (`NoSharedTopic`).
- Measure constructors return `Result<Self, MeasureParseError>` and sort their cutoffs. `RbpMeasure::new` and `RbpResidMeasure::new` read `p` from the parameter string. `GainsConfig::parse` returns a `Result`.
- `resolve_measures` returns `ResolvedMeasures`, which holds the measures and the warnings; `in_trec_eval_order` orders measures as `trec_eval` prints them.
- `QrelsData`, `QrelsJGData` and `RunData` have a `warnings` field. `EvalConfig` has an `all_query_values` field and a `validate` method.
- `ParseError` has the variants `NoDataLines`, `NulCharacter`, `CarriageReturn` and `InvalidUtf8`, and the `expected` field of `InvalidFieldCount` is a `FieldCount`.
- Added: `Measure::reads_ranking`, which defaults to `true`; `DcgMeasure`; `check_rel_level`, `REL_LEVELS` and `read_leading_integer`; and `repeated_document_lines`.

### Python

- `Evaluator` and `evaluate` refuse a negative relevance level, check grades, repeated documents and NaN scores as the file readers do, and raise `NoSharedTopicError`, a subclass of `ValueError`, when the run shares no topic with the qrels and `complete_set_average` is false (#80, #167, #177). Warnings are issued as `UserWarning`, at most ten for one input, then one that counts the rest.
- The pytrec_eval wrapper keeps unjudged documents by default, returns every value as a float, returns per-topic `gm_map`, `gm_bpref`, `num_q` and `runid`, and leaves out a topic whose judgments are an empty mapping, as pytrec_eval does (#199). [docs/python/compat.md](docs/python/compat.md) describes the effect on results.
