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

Besides the input and parameter differences listed below, two exit statuses differ from `trec_eval` 10.0. Without `-c`, `trec_eval` also exits `4` when any qrels topic has no results; `te-rust` scores the shared topics. `trec_eval` checks some measure parameters only while scoring and exits `4` for them, as for `utility.1,2`, where `te-rust` exits `2`.

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
| `-l` | `<int>` | Minimum relevance level at which a document is considered relevant. Must be 0 or more. Documents with judgments $< l$ are considered non-relevant. Read as its [leading integer](#integer-values). | `1` |
| `-c` | | **Complete set average**: Average over the complete set of queries present in the qrels file rather than the intersection of queries evaluated in the run. It evaluates each qrels topic missing from the run as an empty ranking. | Disabled |
| `-J` | | **Judged docs only**: Remove all unjudged documents from the retrieved ranking before evaluating. | Disabled |
| `-M` | `<int>` | Maximum number of retrieved documents per topic to evaluate. Read as its [leading integer](#integer-values). | No limit |
| `-N` | `<int>` | Number of documents in the collection. Only `utility` reads it, when its fourth coefficient is not zero. Read as its [leading integer](#integer-values). | `0` |
| `-m` | `<measure>` | Calculate only the indicated measure or measure group (e.g. `-m map`, `-m P.5,10`, `-m official`). Repeatable. When two requests give parameters for one measure, the first list is used, as in `trec_eval`, and `te-rust` warns that the other is ignored. An integer cutoff is read as its [leading integer](#integer-values), so `-m P.5x` gives `P_5` with a warning. | `official` for `qrels`; `qrels_jg` for `qrels_jg` |
| `-R` | `--rel-format <format>` | Format of the relevance judgments file: `qrels` or `qrels_jg`. | `qrels` |
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
  A grade is read as its [leading integer](#integer-values). `te-rust` refuses a grade with no leading integer, such as `.5` or `abc`, with exit status `2`; `trec_eval` 10.0 reads it as 0.

Example `qrels.txt`:
```text
301 0 FBIS3-10082 1
301 0 FBIS3-10169 0
301 0 FBIS3-10243 2
302 0 LA010189-0001 1
```

Blank lines are skipped with a warning that names the line. `trec_eval` stops at a blank line in a qrels file with "Malformed line" and exits 2. `te-rust` prints its warnings about a file only when it reads the whole file without error.

A topic whose judgments are all negative has no relevant document, and `te-rust` scores its ranking as such. When that topic sorts first, `trec_eval` 10.0 exits `4` with "Can't calculate measure" for every measure except `num_q`. When it sorts later, `trec_eval` 10.0 uses the grade counts of the topic before it, and with grades of -2 or lower it can crash.

Lines that start with `#` are comments and are ignored. Comments in qrels files were a trec_eval 10.x feature to support documenting how a qrels file should be use to reproduce published scores. Leading ASCII whitespace before `#` is allowed. `trec_eval` 10.0 accepts a qrels comment only when `#` is in the first column; an indented comment such as `  # comment` gives "Malformed line" and exit status `2`. When the last line of a qrels file is a comment without a newline, `trec_eval` 10.0 reads past the end of the file and can crash or report "Malformed line"; `te-rust` ignores the comment.

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
  `trec_eval` reads the score with `atof`, which parses as `strtod` does. As in `trec_eval`, `inf`, `-inf` and hexadecimal values such as `0x1p3` are scores. A score outside the range of a double is read as `strtod` reads it, `1e400` as inf and `1e-400` as 0, with a warning on stderr from `te-rust`. A score with characters after the number, such as `1.5x`, is read as its leading number, 1.5; `te-rust` warns on stderr. A score such as `1e400x` gets both warnings. `trec_eval` 10.0 performs both conversions silently. NaN, or a score with no leading number, is an error. `trec_eval` 10.x reads a score with no leading number as 0, and it ranks a NaN score in an undefined order.
- `run_id`: System run label (`str`).

Example `run.txt`:
```text
301 Q0 FBIS3-10082 1 14.567 my_bm25_run
301 Q0 FBIS3-10243 2 12.340 my_bm25_run
301 Q0 FBIS3-10169 3  9.812 my_bm25_run
```

Lines starting with `#` in either file are treated as comments and ignored. Comments in run files were a trec_eval 10.x feature to support including metadata and documentation about a run.

Lines in either file end with LF or CRLF. A NUL byte, or a CR that is not followed by LF, as in a file with CR line endings, is an error that names the line. A CR that ends the file is accepted. A data line must be UTF-8, and the error names the first line that is not; `trec_eval` 10.x reads such lines. A comment line may hold bytes that are not UTF-8, and a NUL or a lone CR in a comment line is an error too. `trec_eval` 10.x reads a file with CR line endings as one line, and it accepts a lone CR in a comment line. A UTF-8 byte-order mark at the start of a file is skipped. `trec_eval` 10.x reads the mark as part of the first qid. Of the errors found while it reads the lines of a file, `te-rust` reports the one in the first line; a document repeated in a `qrels_jg` file is found after the whole file is read. `trec_eval` with the fixes for issue #143 checks the whole file for NUL bytes and lone CRs before it reads any field, so it reports a NUL byte on line 2 before an unreadable grade on line 1.

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
- Measure parameters other than integer cutoffs in a form that only `trec_eval` 10.0 reads: with characters after the number, as in `set_F.0.5x` or `relstring.5x`, or in hexadecimal, as in `dcg.1=0x1p0` or `rbp.p=0x1p-1`. `trec_eval` 10.0 reads the leading number silently. `te-rust` refuses these parameters with exit status `2`. It also refuses negative, non-integer or out-of-range `relstring` lengths. With retrieved grades `1,0,1,0,2`, `trec_eval` 10.0 prints `''` for `relstring.-1`, `'10'` for `relstring.2.5`, and `'10102'` for `relstring.1e300` and `relstring.20000000000000000000`; `te-rust` exits `2` for each. `relstring.5.0` is also refused by `te-rust` and read as length 5 by `trec_eval` 10.0.
- Float parameters that are empty or have no leading number. `trec_eval` 10.0 reads them as 0: `iprec_at_recall.x` prints the row `iprec_at_recall_0.00`, `iprec_at_recall.0.5,` adds the level 0, `utility.1,-1,,0` reads a third coefficient of 0, and `rbp.p=x` sets `p` to 0. `te-rust` refuses each of them with exit status `2`.

### Options, parameters and inputs

- Repeated options: `te-rust` refuses `-q -q` and `-M2 -M1` with exit status `1`. `trec_eval` 10.0 accepts repeated flags and uses the last value of a repeated value option: `-M2 -M1 -m num_ret` evaluates one retrieved document per topic. `-m` is repeatable in both tools.
- Numeric option values: `te-rust` requires the leading integer of `-M` and `-N` to lie between 0 and 9223372036854775807 and exits `1` for a negative or out-of-range value. `trec_eval` 10.0 accepts `-N-1`, `-M9223372036854775808` and `-N9223372036854775808`. It also accepts `-M-1`: `-m num_q` succeeds. With judgments `a=1,b=0` and run order `a,b`, `-M-1 -m map` terminates with SIGSEGV. `te-rust` refuses `-l x`, `-M x` and `-N x` with exit status `1`; `trec_eval` 10.0 reads each value as 0. `te-rust` also refuses a level above 9223372036854775807, which `trec_eval` 10.0 accepts.
- Repeated parameters within one measure: `te-rust` refuses `rbp.p=0.5,p=0.8` with exit status `2`. `trec_eval` 10.0 uses the last `p`. With one relevant document followed by a nonrelevant document, its output is `rbp_p=0.5,p=0.8 all 0.2000`.

`te-rust` trims leading and trailing Unicode whitespace around float cutoffs, gain-map keys and values, the `rbp` and `rbp_resid` key `p`, and the `relstring` length. `trec_eval` 10.0 accepts ASCII whitespace around float cutoffs and lengths. Its treatment of gain-map keys and non-breaking spaces differs. In this table, `NBSP` stands for U+00A0. Parameter lists containing whitespace must be quoted in the shell.

| Parameter | `te-rust` | `trec_eval` 10.0 |
| :--- | :--- | :--- |
| `Rprec_mult.NBSP1` | Row `Rprec_mult_1.00` | Row `Rprec_mult_0.00` |
| `iprec_at_recall.NBSP0.5` | Row `iprec_at_recall_0.50` | Row `iprec_at_recall_0.00` |
| `relstring.NBSP2NBSP` | First two relevance characters | `''` |
| `ndcg. 1=3`, `ndcg.1 =3` | `1.0000` | `0.8597` |
| `ndcg.NBSP1=3`, `ndcg.1NBSP=3` | `1.0000` | `0.8597` |
| `ndcg.1=NBSP3` | `1.0000` | `0.6309` |
| `rbp. p=0.5`, `rbp.p =0.5` | `p` is 0.5: `0.5000` | ` p` is read as a gain level and `p` stays 0.9: `0.1400` |
| `set_F. 0.5` | Exit `2`; `set_F` does not trim its parameter | Accepted |

The `ndcg` and `rbp` values above use judgments `a=1,b=2` and run order `a,b`. Both tools accept ASCII whitespace around a gain value, as in `ndcg.1= 3 `, and give `1.0000` on that input.

### Integer values

Both tools read an integer value with characters after its leading integer, such as `1.5`, `0.9` or `5x`, as that integer: 1, 0 or 5. This applies to the values of `-l`, `-M` and `-N`, to integer cutoffs in `-m`, and to grades in `qrels` and `qrels_jg` files. `te-rust` warns on stderr; `trec_eval` 10.0 reads the value silently.

### Judgment groups

Without `-m`, `te-rust` selects `official` for `-R qrels` and `qrels_jg` for `-R qrels_jg`. `trec_eval` 10.0 selects `official` for both; with `-R qrels_jg` it exits `4` because that group contains measures that require standard qrels.

`te-rust` also accepts measures for the other judgments format. A standard measure such as `map` on `qrels_jg` uses the first judgment group of each topic. A judgment-group measure such as `map_avgjg` on standard `qrels` treats each topic as one group. `trec_eval` 10.0 refuses both requests with exit status `4`.

For example, with judgments `1 a a 0`, `1 a b 1`, `1 b a 1`, `1 b b 0`, and run order `a,b`, `-R qrels_jg -m map -m map_avgjg` prints:

```text
map                   all 0.5000
map_avgjg             all 0.7500
```

With standard judgments `a=1,b=0` and the same run, `-R qrels -m map_avgjg` prints `map_avgjg all 1.0000`.

### `te-rust` extensions

`trec_eval` refuses each of these with "illegal measure" and exit status 1.

- Cutoffs written after an underscore: `P_5`, and likewise `P_avgjg_5`, `recall_10`, `ndcg_cut_10`, `map_cut_100`, `success_5` and `unj_10`. `P_5` prints the same row as `P.5`.
- Parameters written after `@` instead of a period, including `rbp@p=0.5`, which prints `rbp_p=0.5 all 0.5000` for a relevant first document. Cutoff examples include `P@5`, `recall@10`, `success@5`, `unj@10` and `relative_P@5`. `ndcg@10` and `map@100` mean `ndcg_cut.10` and `map_cut.100`.
- The names `mrr` and `rr` for `recip_rank`, and `precision` for `P`, and measure and group names in any case, such as `MAP`.
- The measure `ndcg_p`. The `trec_eval` source includes `m_ndcg_p.c`, but `trec_eval` 10.x does not register or build it, so `-m ndcg_p` fails there.

`te-rust` accepts the `-R` format names `qrels` and `qrels_jg` in any case. `trec_eval` 10.0 refuses `-R QRELS` and `-R QrElS_Jg` with exit status `2`. The long spelling `--rel-format` is also a `te-rust` extension; `trec_eval` 10.0 refuses it with exit status `1`.

`te-rust` also reads the value of a short option after `=`, as in `-l=2`, `-M=100`, `-N=10`, `-m=map` or `-R=qrels`, as if it were a separate argument. `trec_eval` reads the `=` as part of the value. It refuses `-m=map` and `-R=qrels`. `trec_eval` 10.0 reads `-l=2`, `-M=100` and `-N=10` as 0 without a message.

The confidence-interval options, `--help-measures`, `--help-measure` and `--global-gains` are also `te-rust`'s own.

### Cutoff row names and ordering

`te-rust` prints the full `Rprec_mult` multiple with two decimal places. For `-m Rprec_mult.100000000`, its row name is `Rprec_mult_100000000.00`; `trec_eval` 10.0 truncates it to `Rprec_mult_10000000`.

`te-rust` sorts integer cutoffs in ascending numerical order. `trec_eval` 10.0 can order cutoffs above `INT_MAX` (2147483647) differently: `-m P.4294967297,1` prints `P_1` then `P_4294967297` in `te-rust`, and `P_4294967297` then `P_1` in `trec_eval` 10.0.

### Complete set averaging (`-c`)

With `-c`, `te-rust` sums `num_rel` using `-l`. `trec_eval` 10.0 recounts the summary using level 1, even when every qrels topic has results. Per-topic `num_rel` values still use `-l` in both tools.

For a qrels topic missing from the run, `te-rust` evaluates an empty ranking. `trec_eval` 10.0 evaluates a one-document placeholder ranking and then resets selected values. Explicit parameters change row names, so the resets for `utility`, `rbp_resid` and `relstring` no longer apply. A gain map assigning gain to level `-1` can also give the placeholder a positive score in `dcg`, `ndcg`, `ndcg_rel`, `Rndcg`, `G` and `rbp`.

For these files:

```text
# qrels.txt
1 0 a 1
1 0 b 2
2 0 a 2

# run.txt
1 Q0 a 1 2 r
1 Q0 b 2 1 r
```

The following commands, followed by `qrels.txt run.txt`, give these values. Both tools exit `0`.

| Options | Row | `te-rust` | `trec_eval` 10.0 |
| :--- | :--- | :--- | :--- |
| `-c -l2 -m num_rel` | `num_rel all` | `2` | `3` |
| `-c -q -m utility.1,-1,0,0` | `utility_1,-1,0,0 2` | `0.0000` | `-1.0000` |
| `-c -q -m rbp_resid.p=0.5` | `rbp_resid_p=0.5 2` | `0.0000` | `1.0000` |
| `-c -q -m relstring.3` | `relstring_3 2` | `''` | `'-'` |
| `-c -q -m dcg.-1=1` | `dcg_-1=1 2` | `0.0000` | `1.0000` |
| `-c -q -m ndcg.-1=1` | `ndcg_-1=1 2` | `0.0000` | `0.5000` |
| `-c -q -m ndcg_rel.-1=1` | `ndcg_rel_-1=1 2` | `0.0000` | `0.5000` |
| `-c -q -m Rndcg.-1=1` | `Rndcg_-1=1 2` | `0.0000` | `0.5000` |
| `-c -q -m G.-1=1` | `G_-1=1 2` | `0.0000` | `0.3155` |
| `-c -q -m rbp.p=0.5,-1=1` | `rbp_p=0.5,-1=1 2` | `0.0000` | `0.2500` |

Without explicit parameters, both tools give topic 2 a value of `0.0000` for `utility` and `rbp_resid`, and `''` for `relstring`. Adding `-J` removes the placeholder and makes the parameterized `utility`, `rbp_resid`, `relstring` and `ndcg` examples above agree too.

### Gain calculations that differ from bundled `trec_eval` 10

- `te-rust` sorts gains by their full floating-point values. The bundled C comparator converts the gain difference to an integer, treating absolute differences below 1 as equal. With judgments `a=1, b=0`, run order `a,b`, and gains `1=1,0=0.9`, `-m ndcg.1=1,0=0.9` gives `1.0000` in `te-rust` and `1.0241` in the bundled C.
- `Rndcg` returns 0 when no judged document has a positive gain. With the same judgments and run order `a,b`, `-m Rndcg.0=0,1=-1` gives `0.0000` in `te-rust` and `nan` in the bundled C.
