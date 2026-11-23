"""
Drop-in compatibility layer for `pytrec_eval`.

Existing scripts using `pytrec_eval` can switch to `te-rust` simply by:
```python
import trec_eval.compat.pytrec_eval as pytrec_eval
```
"""

import collections
import io
import re
from decimal import Decimal
from typing import Any, Dict, Iterable, List, Set, Union
from trec_eval._trec_eval import Evaluator, NoSharedTopicError, __version__


# Measures the wrapper evaluates by name: pytrec_eval's, and those only te-rust has.
supported_measures: Set[str] = {
    "official", "all_trec", "set",
    "runid", "num_q", "num_ret", "num_rel", "num_rel_ret", "num_nonrel_judged_ret",
    "map", "map_cut", "gm_map", "Rprec", "Rprec_mult", "recip_rank",
    "P", "relative_P", "bpref", "gm_bpref", "binG", "G",
    "ndcg", "dcg", "ideal_dcg", "ndcg_cut", "ndcg_rel", "ndcg_p", "Rndcg",
    "recall", "success", "11pt_avg", "iprec_at_recall", "utility", "relstring",
    "set_P", "set_recall", "set_relative_P", "set_map", "set_F", "infAP", "unj", "rbp",
    "rbp_resid", "yaap",
}

# Measures that print one row per parameter value. pytrec_eval merges the
# parameter lists of repeated requests, such as P_5 and P.10, reading each
# leading numeric list and ignoring any trailing characters. The wrapper merges
# them only for these measures and passes other requests to te-rust as written.
_ROW_PER_VALUE = (
    "P", "recall", "success", "unj", "ndcg_cut", "map_cut", "relative_P",
    "iprec_at_recall", "Rprec_mult",
)
_VALUE_REQUEST = re.compile(
    rf"({'|'.join(_ROW_PER_VALUE)})[_.]([0-9]+(?:\.[0-9]+)?(?:,[0-9]+(?:\.[0-9]+)?)*)"
)


def parse_run(f_or_path: Union[str, io.IOBase, Any]) -> Dict[str, Dict[str, float]]:
    """Parse a TREC run file or file-like stream into a nested dictionary."""
    run: Dict[str, Dict[str, float]] = collections.defaultdict(dict)

    if isinstance(f_or_path, str) and "\n" not in f_or_path and len(f_or_path) < 1024:
        with open(f_or_path, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                parts = line.strip().split()
                if len(parts) >= 6 and not parts[0].startswith("#"):
                    qid, docno, score = parts[0], parts[2], float(parts[4])
                    run[qid][docno] = score
        return dict(run)

    # File-like or text content
    lines = f_or_path.splitlines() if isinstance(f_or_path, str) else f_or_path
    for line in lines:
        line_str = line.decode("utf-8") if isinstance(line, bytes) else str(line)
        parts = line_str.strip().split()
        if len(parts) >= 6 and not parts[0].startswith("#"):
            qid, docno, score = parts[0], parts[2], float(parts[4])
            run[qid][docno] = score

    return dict(run)


def parse_qrel(f_or_path: Union[str, io.IOBase, Any]) -> Dict[str, Dict[str, int]]:
    """Parse a TREC qrels file or file-like stream into a nested dictionary."""
    qrel: Dict[str, Dict[str, int]] = collections.defaultdict(dict)

    if isinstance(f_or_path, str) and "\n" not in f_or_path and len(f_or_path) < 1024:
        with open(f_or_path, "r", encoding="utf-8", errors="replace") as f:
            for line in f:
                parts = line.strip().split()
                if len(parts) >= 4 and not parts[0].startswith("#"):
                    qid, docno, rel = parts[0], parts[2], int(parts[3])
                    qrel[qid][docno] = rel
        return dict(qrel)

    lines = f_or_path.splitlines() if isinstance(f_or_path, str) else f_or_path
    for line in lines:
        line_str = line.decode("utf-8") if isinstance(line, bytes) else str(line)
        parts = line_str.strip().split()
        if len(parts) >= 4 and not parts[0].startswith("#"):
            qid, docno, rel = parts[0], parts[2], int(parts[3])
            qrel[qid][docno] = rel

    return dict(qrel)


class RelevanceEvaluator:
    """Drop-in replacement for pytrec_eval.RelevanceEvaluator."""

    def __init__(
        self,
        qrel: Any,
        measures: Union[Set[str], Iterable[str], str],
        relevance_level: int = 1,
        judged_docs_only_flag: bool = False,
    ):
        measure_names = [measures] if isinstance(measures, str) else list(measures)
        native_measures: List[Any] = []
        values: Dict[str, Dict[Decimal, str]] = collections.defaultdict(dict)
        for measure in measure_names:
            match = _VALUE_REQUEST.match(measure) if isinstance(measure, str) else None
            if match is None:
                native_measures.append(measure)
                continue
            for value in match[2].split(","):
                # Decimal keys merge spellings of one value, such as 5, 05 and 5.0.
                values[match[1]].setdefault(Decimal(value), value)
        native_measures += [f"{name}.{','.join(v.values())}" for name, v in values.items()]
        if isinstance(qrel, dict):
            # pytrec_eval drops a topic whose judgments are empty. len() raises
            # TypeError for None, as in pytrec_eval.
            qrel = {
                qid: judgments for qid, judgments in qrel.items() if len(judgments) > 0
            }

        self._evaluator = Evaluator(
            qrels=qrel,
            measures=native_measures,
            relevance_level=relevance_level,
            judged_docs_only=judged_docs_only_flag,
            all_query_values=True,
        )
        self.measures = set(measure_names)

    def evaluate(self, run: Any) -> Dict[str, Dict[str, float]]:
        """Evaluate run and return nested dict matching pytrec_eval output format."""
        try:
            eval_result = self._evaluator.evaluate(run)
        except NoSharedTopicError:
            # pytrec_eval returns {} for a run that shares no topic with the qrels.
            return {}
        # pytrec_eval gives every value as a float, and 0.0 for runid and relstring.
        return {
            qid: {
                name: 0.0 if isinstance(value, str) else float(value)
                for name, value in scores.items()
            }
            for qid, scores in eval_result.to_dict().items()
        }


__all__ = [
    "RelevanceEvaluator",
    "parse_run",
    "parse_qrel",
    "supported_measures",
    "__version__",
]
