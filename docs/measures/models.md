# Summary Models, User Metrics & Diagnostics

---

## Geometric Mean Measures (`gm_map`, `gm_bpref`)

Geometric mean averaging across topic queries prevents systems from achieving high overall scores by performing well on a small number of easy queries while failing on difficult queries.

### Geometric Mean of MAP (`gm_map`)

For topic average precision scores $\text{AP}_q \in [0, 1]$:

$$\text{gm\_map} = \exp\left( \frac{1}{|Q|} \sum_{q \in Q} \ln\left( \max(\text{AP}_q, \epsilon) \right) \right)$$

where $\epsilon = 0.00001$ is the minimum score floor preventing $\ln(0)$.

- Usage: `-m gm_map`
- Reference: Robertson, S. (2006). "On GMAP: and other transformations of information retrieval metrics." *Proceedings of the 15th ACM CIKM*, pp. 78–83.

### Geometric Mean of Bpref (`gm_bpref`)

$$\text{gm\_bpref} = \exp\left( \frac{1}{|Q|} \sum_{q \in Q} \ln\left( \max(\text{bpref}_q, \epsilon) \right) \right)$$

---

## Rank-Biased Precision (`rbp`, `rbp_resid`)

Models a user browsing a ranked list who examines the document at rank $k$ with persistence probability $p^{k-1}$, stopping with probability $1 - p$.

### Definition

$$\text{RBP} = (1 - p) \sum_{k=1}^N \text{rel}(k) \cdot p^{k-1}$$

where $\text{rel}(k)$ is the gain for the document's relevance level. The gains come from the measure's own gain map when it has one, and otherwise from `--global-gains`. The gain table holds the levels that map names and every level from 0 to the highest judged grade; a level in that range that the map omits has its relevance level as its gain. The normalization range includes 0: $g_{\min} = \min(0, \min_g g)$ and $g_{\max} = \max(0, \max_g g)$ over the gain table. If $g_{\min} < 0$ or $g_{\max} > 1$, each table entry $g$ becomes $(g - g_{\min}) / (g_{\max} - g_{\min})$. Otherwise the gains are unchanged. A level outside the table, including the unjudged sentinel by default, has gain 0.

- Usage: `-m rbp`, `-m rbp.p=0.8`, or with a gain map, `-m rbp.p=0.8,1=1,2=3`.

### RBP Residual (`rbp_resid`)

For a ranking of $N$ retrieved documents, let $U$ be the set of ranks that hold an unjudged document:

$$\text{RBP}_{\text{resid}} = (1 - p) \sum_{k \in U} p^{k-1} + p^N \quad \text{for } U \neq \emptyset, \text{ and } 0 \text{ otherwise}$$

This is the formula of `trec_eval`'s `m_rbp_resid.c`. Moffat and Zobel include $p^N$ for every ranking, also one without an unjudged document.

The first term is the weight of unjudged retrieved documents, and $p^N$ is the weight of the ranks after $N$. The interpretation as a possible increase in RBP assumes that unjudged retrieved documents have gain 0 in RBP. `rbp_resid` ignores gain maps and can overestimate that increase otherwise. For judgments `a=1,b=0` and run order `x,a`, both `rbp.p=0,-1=1` and `rbp_resid.p=0,-1=1` give `1.0000`, although RBP is already at its maximum.

- Parameter: `p` (persistence parameter, default `0.9`). A gain map is checked as for `rbp` and not used.
- Usage: `-m rbp_resid` or `-m rbp_resid.p=0.8`.
- With `-c`, parameterized `rbp_resid` can differ from `trec_eval` 10.0 on missing topics; see [Complete set averaging](../cli.md#complete-set-averaging--c).
- Reference: Moffat, A., & Zobel, J. (2008). "Rank-biased precision for measurement of retrieval effectiveness." *ACM Transactions on Information Systems (TOIS)*, 27(1): Article 2.

---

## 11-Point Interpolated Precision (`11pt_avg`, `iprec_at_recall`)

Interpolated precision evaluated at 11 standard recall levels: $0.0, 0.1, 0.2, \dots, 1.0$.

### Definition

Interpolated precision at recall level $r$:

$$P_{\text{interp}}(r) = \max_{r' \ge r} P(r')$$

- `11pt_avg`: Arithmetic mean of the 11 interpolated precision levels. Levels given as parameters, as in `-m 11pt_avg.1.0,0.5`, are sorted first. `trec_eval` 10.x keeps their order, and its value then depends on that order.
- `iprec_at_recall`: Reports the 11 individual interpolated precision scores.
- Usage: `-m 11pt_avg` or `-m iprec_at_recall`, or with recall levels, `-m iprec_at_recall.0.25,0.5`. Each level must lie in $[0, 1]$.

---

## Linear Utility (`utility`)

Assigns explicit utility weights $a, b, c, d$ to the four cells of the $2 \times 2$ contingency table:

$$\text{Utility} = a \cdot \text{num\_rel\_ret} + b \cdot \text{num\_nonrel\_ret} + c \cdot \text{num\_rel\_nonret} + d \cdot \text{num\_nonrel\_nonret}$$

- Parameters: `utility.<a,b,c,d>` (default: `1.0, -1.0, 0.0, 0.0`).
- The fourth coefficient uses `-N <num_docs_in_coll>`, which defaults to 0. A collection size smaller than the counted documents gives a negative fourth cell. With five retrieved, judged documents, `-m utility.0,0,0,1` gives `-5.0000` without `-N` in both tools; `-N10` gives `5.0000`.
- Usage: `-m utility.1,-1,0,0`
- Parameterized `utility` can differ from `trec_eval` 10.0 on topics missing from the run with `-c`; see [Complete set averaging](../cli.md#complete-set-averaging--c).

---

## Diagnostic Measures

### Relevance String (`relstring`)
Constructs a string representation of relevance judgments for the top $k$ retrieved documents. Retrieved grades `1,0,1,0,2` give `'10102'` in both tools. `-` denotes a retrieved document absent from the qrels; `.` denotes a retrieved document present in the qrels with a negative grade.

- Parameter: cutoff length $k$ (default `10`).
- Usage: `-q -m relstring` or `-q -m relstring.20`. The string is printed only for individual topics.
- With `-c`, parameterized lengths can differ from `trec_eval` 10.0 on missing topics; see [Complete set averaging](../cli.md#complete-set-averaging--c).

### Run Identifier (`runid`)
Reports the system run label recorded in column 6 of the run results file.

- Usage: `-m runid`
