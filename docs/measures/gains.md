# Gain-Based & Discounted Measures

Gain-based measures like NDCG evaluate rankings using graded relevance judgments where higher relevance grades contribute greater utility, discounted by rank position.

---

## Discounted Cumulative Gain at Cutoffs (`ndcg_cut`, `ndcg`)

Normalized Discounted Cumulative Gain (NDCG) measures the ratio of actual Discounted Cumulative Gain to the ideal Discounted Cumulative Gain (IDCG) obtained by sorting positive-gain judged documents by gain.

### Mathematical Formulation

$$\text{DCG}@k = \sum_{i=1}^k \frac{\text{gain}(\text{rel}_i)}{\log_2(i + 1)}$$

$$\text{NDCG}@k = \begin{cases} \frac{\text{DCG}@k}{\text{IDCG}@k} & \text{if } \text{IDCG}@k > 0 \\ 0.0 & \text{otherwise} \end{cases}$$

where:
- $\text{gain}(r)$ is the gain assigned to relevance grade $r$. In standard `trec_eval`, the default gain is the integer relevance level: $\text{gain}(r) = r$.
- $\text{IDCG}@k$ is the DCG computed on the ideal ranking formed by sorting positive-gain documents in the qrels by gain in descending order.

### Cutoffs (`ndcg_cut`)
- Default rank cutoffs: $5, 10, 15, 20, 30, 100, 200, 500, 1000$.
- Usage: `-m ndcg_cut.10,20` or `ndcg@10`.

### DCG and Ideal DCG (`dcg`, `ideal_dcg`)
`dcg` is the DCG of the whole ranking and `ideal_dcg` the DCG of the ideal ranking, the numerator and the denominator of `ndcg`. The ideal ranking holds the judged documents with a positive gain, in descending order of gain. `dcg` also counts negative gains. Each summary is the mean of the topic values, so the mean of `ndcg` is not the ratio of the other two means.

- Usage: `-m dcg`, `-m ideal_dcg`, or `-m dcg.1=3,2=9` for custom gains, which work as for `ndcg`.

---

## NDCG Variants

### 1. Original Discount (`ndcg_p`)
Computes NDCG with the gains of `ndcg`, the relevance level by default, and the original discount of Järvelin and Kekäläinen (2002): rank 1 is not discounted, and rank $i \ge 2$ is divided by $\log_2 i$.

$$\text{DCG}_p = \text{gain}(\text{rel}_1) + \sum_{i=2}^{n} \frac{\text{gain}(\text{rel}_i)}{\log_2 i}$$

`ndcg` divides rank $i$ by $\log_2(i + 1)$, so the two differ from rank 2 on. For grades 1, 0, 2 at ranks 1 to 3, the ideal order is 2, 1, 0, and

$$\text{ndcg\_p} = \frac{1 + 2 / \log_2 3}{2 + 1} = 0.7540, \qquad \text{ndcg} = \frac{1 + 2 / \log_2 4}{2 + 1 / \log_2 3} = 0.7602.$$

Gains may be zero or negative, and level 0 may be given a gain. `ndcg_p` is 0 when the run retrieves no relevant document. `trec_eval` 10.x ships `m_ndcg_p.c` but does not register it; see [Differences from `trec_eval`](../cli.md#differences-from-trec_eval).

- Usage: `-m ndcg_p`, or `-m ndcg_p.1=3.5,2=9.0` for custom gains

### 2. NDCG Averaged over Relevant Documents (`ndcg_rel`)
Uses graded gains, with relevance defined by positive gain rather than `-l`. At each retrieved document with positive gain, add the NDCG of that ranking prefix. Let $R$ count positive-gain judged documents and $r$ count positive-gain retrieved documents, including documents at sentinel levels assigned positive custom gains. Add $(R - r)$ times the final retrieved DCG divided by the full ideal DCG to the prefix sum. This correction can be negative. Divide by $R$; return 0 if $R$ or the corrected sum is not positive.

- Usage: `-m ndcg_rel`

### 3. R-NDCG (`Rndcg`)
Averages DCG/IDCG at the boundaries where the ideal ranking changes gain level, including the transition from positive gain to zero. Missing retrieved ranks contribute zero gain. If the run extends beyond the first zero-gain ideal rank, include the ratio at the end of the run too. Return 0 when no boundary has positive ideal DCG, or no judgment meets `-l`.

- Usage: `-m Rndcg`

---

## Normalized Gain (`G`, `binG`)

### Normalized Gain (`G`)
Let $g_i$ be the retrieved gain at rank $i$, and $h_i$ the ideal gain at that rank, using only positive judged gains and padding with zeros. Define cumulative retrieved gain $S_i = \sum_{j=1}^i g_j$ and cumulative cost $C_i = \sum_{j=1}^i \max(1, h_j)$. Then

$$\text{G} = \frac{1}{\sum_j h_j} \sum_{i:g_i \ne 0} \frac{g_i}{\log_2(2 + C_i - S_i)}.$$

The denominator sums all positive judged gains. Return 0 when it is zero.

### Binary Normalized Gain (`binG`)
For each retrieved relevant document, let $n_i$ count the preceding documents whose grade is below $l$. With $R$ relevant judged documents,

$$\text{binG} = \frac{1}{R} \sum_{i:\text{rel}_i \ge l} \frac{1}{\log_2(2 + n_i)}.$$

Return 0 when $R = 0$. Both measures score the whole ranking and have no cutoff family.

---

## Custom Relevance-to-Gain Mappings

Custom gain mappings can be passed globally via `--global-gains` (CLI) / `global_gains` (Python), or directly to the measure parameter string:

### Syntax
- CLI flag: `--global-gains 1=1.0,2=3.0,3=7.0,4=15.0`
- Metric parameter: `-m ndcg.1=1.0,2=3.0,3=7.0`
- Python API: `evaluator = Evaluator(qrels, measures=["ndcg@10"], global_gains="1=1.0,2=3.0")`

### Rules
- Each entry is `<level>=<gain>`. The level is an integer, and the gain a finite number.
- A level may appear once.
- `te-rust` refuses a map that breaks these rules, with a message. `trec_eval` 10.x skips a name that is not an integer. For a level given twice, it can score the run with one gain and the ideal ranking with the other. It reads each gain with `atof`, which reads `x` as 0 and `2x` as 2.
