# Ecosystem Compatibility Layers

`trec-eval` includes drop-in compatibility modules for existing Python IR libraries.

---

## 1. `pytrec_eval` Compatibility

Scripts written for `pytrec_eval` can switch to `trec-eval` by updating the import:

```python
# Replace: import pytrec_eval
import trec_eval.compat.pytrec_eval as pytrec_eval

qrel = {
    'q1': {'d1': 0, 'd2': 1, 'd3': 0},
    'q2': {'d2': 1, 'd3': 1},
}

run = {
    'q1': {'d1': 1.0, 'd2': 0.0, 'd3': 1.5},
    'q2': {'d1': 1.5, 'd2': 0.2, 'd3': 0.5},
}

# Construct RelevanceEvaluator with measure set
evaluator = pytrec_eval.RelevanceEvaluator(qrel, {'map', 'ndcg'})

# Returns exact nested dictionary format matching pytrec_eval
# (an empty dict when the run shares no topic with qrel, as in pytrec_eval)
results = evaluator.evaluate(run)
# {
#     'q1': {'map': 0.3333333333333333, 'ndcg': 0.5},
#     'q2': {'map': 0.5833333333333333, 'ndcg': 0.6934264036172708}
# }
```

pytrec_eval itself gives other output in some cases:

- It ranks documents by their scores rounded to single precision, so scores that differ only beyond that precision tie and are ordered by document number. The wrapper reads scores as `trec_eval` 10.0 does.
- pytrec-eval-terrier follows `trec_eval` 9.0.8, so it has no `rbp`, `rbp_resid`, `unj` or `ndcg_p`.
- It computes `iprec_at_recall` and `11pt_avg` as `trec_eval` 9.0.8 does.
- It names the rows of a parameterized `relstring`, `11pt_avg`, `set_F` or `utility` without the parameters: `relstring` where the wrapper gives `relstring_4`.
- It stops the Python process on a gain map such as `ndcg.1=2`.
- It reads only the leading digits of a parameter: `set_F.1e1` as 1 and `utility.1,-1,0,0` as one coefficient, where the wrapper reads 10 and four coefficients.
- It reads `set_F_0.5` as `set_F.0.5`. The wrapper reads a request written with an underscore, such as `P_5`, only for the measures that print a row per value. It passes other requests to `te-rust` as written, so it refuses `set_F_0.5`.

### Changes from earlier releases of the wrapper

Earlier releases differed from pytrec_eval in these ways, and the fixes change the results of unchanged code:

- `judged_docs_only_flag` defaults to `False`, as in pytrec_eval; it defaulted to `True`. A topic whose ranking holds unjudged documents now scores differently: with the qrels `{'q': {'a': 1, 'b': 0}}` and a run that ranks an unjudged document first, then `a` and `b`, `map` is 0.5 where it was 1.0. Pass `judged_docs_only_flag=True` for the earlier results.
- Every value is a float, as in pytrec_eval: `relstring` is 0.0 where it was a string, and counts such as `num_ret` are floats where they were integers.
- The per-topic values of `gm_map`, `gm_bpref`, `num_q` and `runid` are returned.
- A topic whose judgments are an empty mapping is left out of the results, as pytrec_eval leaves it out.
- `supported_measures` lists every measure name that pytrec_eval lists.

### Parsing Utilities

```python
# Parse standard files into nested dictionaries
qrels_dict = pytrec_eval.parse_qrel("qrels.txt")
run_dict = pytrec_eval.parse_run("run.txt")
```

---

## 2. `ir_measures` Provider

`trec_eval.compat.ir_measures` provides a provider adapter for the `ir_measures` library:

```python
import ir_measures
from ir_measures import AP, RR, nDCG, P
from trec_eval.compat.ir_measures import TeRustProvider

# Register te-rust as an evaluation provider
ir_measures.register_provider(TeRustProvider())

# Evaluate using standard ir_measures workflows
results = ir_measures.calc_aggregate(
    [AP, nDCG@10, RR, P@5],
    qrels,
    run,
    provider="te_rust",
)
```
