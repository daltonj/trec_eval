# Command-Line Interface Reference

`te-rust` is the standalone command-line evaluation tool. It accepts standard TREC relevance judgments (qrels) and system run results files, evaluates requested measures, and prints relational summary outputs.

It is meant as a drop-in replacement for `trec_eval` 10.x: it reads the same files and takes the same short options and measure specifications, with the exceptions listed in [Differences from `trec_eval`](#differences-from-trec_eval). A difference that concerns one measure, one file format or the output is described with it, in the measure pages or in the sections of this page. If you find an incompatibility with trec_eval 10.x that these pages do not describe, that is a bug on our part.

---

## Exit status

| Status | Meaning |
| :--- | :--- |
| `0` | Success. |
| `1` | Usage error or illegal measure name. |
| `2` | Unreadable or malformed input, or an unusable measure parameter. |
| `4` | A topic cannot be scored: a document repeats within a topic that is scored, or, without `-c`, no run topic is in the qrels. |
| `8` | The output cannot be written, as on a full disk. |

As in `trec_eval`, a repeated document is refused only in a topic that is scored, and only when a requested measure reads the ranking, which every measure but `num_q` and `runid` does. A scored topic is a qrels topic that the run has, or any qrels topic with `-c`. In the run, only the first `-M` documents of the ranking are checked. A duplicate in a `qrels_jg` file is refused when the file is read, as in `trec_eval`, and exits `2`.

Two statuses differ from `trec_eval` 10.x. Without `-c`, `trec_eval` also exits `4` when any qrels topic has no results; `te-rust` scores the shared topics. `trec_eval` checks some measure parameters only while scoring and exits `4` for them, as for `utility.1,2`, where `te-rust` exits `2`.

As in `trec_eval`, an illegal measure name is reported before a file error, and a qrels error before a run error. A repeated document in a qrels file is reported after the run is read, as `trec_eval` finds it while scoring. `te-rust` checks measure parameters before the files and `trec_eval` after them, so with an unusable parameter and a bad file the two report different problems, both with status `2`.

---

## Syntax

```bash
te-rust [OPTIONS] <QRELS_FILE> <RUN_FILE>
```

### Positional Arguments

| Argument | Description |
| :--- | :--- |
| `<QRELS_FILE>` | Path to the relevance judgments file. |
| `<RUN_FILE>` | Path to the system retrieval results (run) file. |

---

## Options and Flags

### Evaluation Flags

| Flag | Option | Description | Default |
| :--- | :--- | :--- | :--- |
| `-q` | | Print evaluations for each individual query/topic in addition to the summary averages. | Disabled |
| `-n` | | Suppress printing of the summary averages/totals at the end. | Disabled |
| `-l` | `<int>` | Minimum relevance level at which a document is considered relevant. Must be 0 or more. Documents with judgments $< l$ are considered non-relevant. As in `trec_eval`, a value such as `1.5` is read as its leading integer, 1, with a warning. | `1` |
| `-c` | | **Complete set average**: Average over the complete set of queries present in the qrels file rather than the intersection of queries evaluated in the run. It evaluates each qrels topic missing from the run as an empty ranking. | Disabled |
| `-J` | | **Judged docs only**: Remove all unjudged documents from the retrieved ranking before evaluating. | Disabled |
| `-M` | `<int>` | Maximum number of retrieved documents per topic to evaluate. Read as `-l` is. | No limit |
| `-N` | `<int>` | Number of documents in the collection. Only `utility` reads it, when its fourth coefficient is not zero. Read as `-l` is. | `0` |
| `-m` | `<measure>` | Calculate only the indicated measure or measure group (e.g. `-m map`, `-m P.5,10`, `-m official`). Repeatable. When two requests give parameters for one measure, the first list is used, as in `trec_eval`, and `te-rust` warns that the other is ignored. An integer cutoff is read as `-l` is, so `-m P.5x` gives `P_5` with a warning. | `official` |
| `-R` | `<format>` | Format of the relevance judgments file: `qrels` or `qrels_jg`. | `qrels` |
| | `--global-gains <gains>` | Custom global relevance-to-gain mapping (e.g. `--global-gains 1=3.5,2=9.0`). | Default gains |

### Bootstrap Confidence Interval Options

| Option | Description | Default |
| :--- | :--- | :--- |
| `-C`, `--ci` | Compute and print bootstrap confidence intervals for summary metrics in relational format (`measure_ci_lower`, `measure_ci_upper`). | Disabled |
| `--ci-pretty` | Format confidence intervals in bracketed human-readable format (`measure all value [lower, upper]`). | Disabled |
| `--ci-alpha <float>` | Significance level $\alpha$ for confidence intervals (e.g. `0.05` for a $95\%$ CI). | `0.05` |
| `--ci-samples <int>` | Number of bootstrap resamples. | `1000` |
| `--seed <int>` | Optional integer seed for deterministic, reproducible bootstrap resampling. | Random |

### Help and Discovery Options

| Option | Description |
| :--- | :--- |
| `--help-measures` | List all available measures with status and a short description, then exit. |
| `--help-measure <name>` | Print detailed mathematical explanation and parameter usage for the specified measure, then exit. |
| `-h`, `--help` | Print CLI usage summary and exit. |
| `-V`, `--version` | Print version information and exit. |

---

## File Formats

### 1. Relevance Judgments File (qrels)

Whitespace-delimited text file with 4 columns:

```text
qid  iter  docno  rel
```

- `qid`: Unique query/topic identifier (`str`).
- `iter`: Iteration/judgment tag (traditionally `0` or accessor ID; ignored during scoring).
- `docno`: Unique document identifier (`str`).
- `rel`: Integer relevance grade from -127 to 127 ($\ge 0$ for judged grades; negative integers reserved for unjudged pool sentinels). A grade outside that range is an error. `trec_eval` 10.x accepts such grades.
  As in `trec_eval`, a grade such as `1.5` is read as its leading integer, 1, with a warning on stderr. A grade with no leading integer, such as `.5`, is an error.

Example `qrels.txt`:
```text
301 0 FBIS3-10082 1
301 0 FBIS3-10169 0
301 0 FBIS3-10243 2
302 0 LA010189-0001 1
```

Blank lines are skipped with a warning that names the line. `trec_eval` stops at a blank line in a qrels file with "Malformed line" and exits 2.

Lines that start with `#` are comments and are ignored. Comments in qrels files were a trec_eval 10.x feature to support documenting how a qrels file should be use to reproduce published scores.

### 2. System Run File

Whitespace-delimited text file with 6 columns:

```text
qid  iter  docno  rank  sim  run_id
```

- `qid`: Unique query/topic identifier (`str`).
- `iter`: Iteration tag (traditionally `Q0`; ignored during scoring).
- `docno`: Unique document identifier (`str`).
- `rank`: Rank position (recomputed during score sorting).
- `sim`: Floating-point retrieval score / similarity (`float`).
  `trec_eval` reads the score with `atof`, which parses as `strtod` does. As in `trec_eval`, `inf`, `-inf` and hexadecimal values such as `0x1p3` are scores. A score outside the range of a double is read as `strtod` reads it, `1e400` as inf and `1e-400` as 0, with a warning on stderr. A score with characters after the number, such as `1.5x`, is read as its leading number, 1.5, with a warning on stderr. NaN, or a score with no leading number, is an error. `trec_eval` 10.x reads a score with no leading number as 0, and it ranks a NaN score in an undefined order.
- `run_id`: System run label (`str`).

Example `run.txt`:
```text
301 Q0 FBIS3-10082 1 14.567 my_bm25_run
301 Q0 FBIS3-10243 2 12.340 my_bm25_run
301 Q0 FBIS3-10169 3  9.812 my_bm25_run
```

Lines starting with `#` in either file are treated as comments and ignored. Comments in run files were a trec_eval 10.x feature to support including metadata and documentation about a run.

Lines in either file end with LF or CRLF. A NUL byte, or a CR that is not followed by LF, as in a file with CR line endings, is an error that names the line. A CR that ends the file is accepted. A data line must be UTF-8, and the error names the first line that is not; `trec_eval` 10.x reads such lines. A comment line may hold bytes that are not UTF-8, and a NUL or a lone CR in a comment line is an error too. `trec_eval` 10.x reads a file with CR line endings as one line, and it accepts a lone CR in a comment line. A UTF-8 byte-order mark at the start of a file is skipped. `trec_eval` 10.x reads the mark as part of the first qid.

---

## Output Formats

On Unix, a closed output pipe terminates `te-rust` through SIGPIPE without a diagnostic, as in `trec_eval`. Other output write failures produce a diagnostic and exit status `8`; `trec_eval` 10.x ignores these failures and exits `0`.

### Standard Relational Output

Scores are printed as tab-separated triples: `measure  topic_id  score`.

```text
runid                 	all	my_bm25_run
num_q                 	all	50
num_ret               	all	50000
num_rel               	all	2279
num_rel_ret           	all	1613
map                   	all	0.2543
Rprec                 	all	0.2812
bpref                 	all	0.2589
recip_rank            	all	0.6124
P_5                   	all	0.4560
P_10                  	all	0.3980
```

When `-q` is enabled, per-topic evaluations precede the `all` summary block.

As in `trec_eval`, measures print in the order of `trec_eval`'s measure table, whatever the order of the `-m` options: `-m map -m num_q` prints `num_q` first. Measures that `trec_eval` lacks, such as `ndcg_p`, print after the others.

### Confidence Interval Output

With `--ci` relational output:
```text
map                   	all	0.2543
map_ci_lower          	all	0.2180
map_ci_upper          	all	0.2915
```

With `--ci-pretty` output:
```text
map                   	all	0.2543 [0.2180, 0.2915]
Rprec                 	all	0.2812 [0.2430, 0.3201]
bpref                 	all	0.2589 [0.2215, 0.2974]
```

---

## Differences from `trec_eval`

### `trec_eval` features that `te-rust` lacks

- A run read from standard input. `trec_eval` reads the run from standard input when the run file is `-`; `te-rust` looks for a file named `-`.
- The options `-T` (results format), `-D` (debug level), `-Z` (z-score file), `-o` (old output format) and `-v` (version). `te-rust` refuses them as unexpected arguments; `-V` prints its version.
- The long options of `trec_eval`, such as `--Max_retrieved_per_topic=5` and `--complete_rel_info_wanted`, other than `--help` and `--version`. `te-rust` refuses them.
- The preference measures (`prefs_simp` and the other `prefs_*` measures, and the `prefs` and `all_prefs` groups), and the `prefs` and `qrels_prefs` formats for `-R`. `te-rust` reports an unknown measure or an unsupported format.
- A negative relevance level. `te-rust` refuses it with exit status 1, whether written as `-l -1`, `-l-1` or `-l=-1`. `trec_eval` 10.0 accepts negative levels and counts documents outside the qrels as relevant while `num_rel` does not, so `map` can exceed 1.
- Measure parameters other than integer cutoffs in a form that only `trec_eval` reads: with characters after the number, as in `set_F.0.5x` or `relstring.5x`, or in hexadecimal, as in `dcg.1=0x1p0` or `rbp.p=0x1p-1`. `trec_eval` reads the leading number, with a warning when characters follow it. `te-rust` refuses these parameters with exit status 2, and also a `relstring` length that is not an integer, such as `relstring.5.0`.

### `te-rust` extensions

`trec_eval` refuses each of these with "illegal measure" and exit status 1.

- Cutoffs written after an underscore: `P_5`, and likewise `P_avgjg_5`, `recall_10`, `ndcg_cut_10`, `map_cut_100`, `success_5` and `unj_10`. `P_5` prints the same row as `P.5`.
- Cutoffs written after `@`: `P@5`, `recall@10`, `success@5`, `unj@10` and `relative_P@5`, and `ndcg@10` and `map@100`, which mean `ndcg_cut.10` and `map_cut.100`.
- The names `mrr` and `rr` for `recip_rank`, and `precision` for `P`, and measure and group names in any case, such as `MAP`.
- The measure `ndcg_p`. The `trec_eval` source includes `m_ndcg_p.c`, but `trec_eval` 10.x does not register or build it, so `-m ndcg_p` fails there.

`te-rust` also reads the value of a short option after `=`, as in `-l=2`, `-M=100`, `-N=10`, `-m=map` or `-R=qrels`, as if it were a separate argument. `trec_eval` reads the `=` as part of the value. It refuses `-m=map` and `-R=qrels`. `trec_eval` 10.0 reads `-l=2`, `-M=100` and `-N=10` as 0 without a message.

The confidence-interval options, `--help-measures`, `--help-measure` and `--global-gains` are also `te-rust`'s own.

### Gain calculations that differ from bundled `trec_eval` 10

- `te-rust` sorts gains by their full floating-point values. The bundled C comparator converts the gain difference to an integer, treating absolute differences below 1 as equal. With judgments `a=1, b=0`, run order `a,b`, and gains `1=1,0=0.9`, `-m ndcg.1=1,0=0.9` gives `1.0000` in `te-rust` and `1.0241` in the bundled C.
- `Rndcg` returns 0 when no judged document has a positive gain. With the same judgments and run order `a,b`, `-m Rndcg.0=0,1=-1` gives `0.0000` in `te-rust` and `nan` in the bundled C.
