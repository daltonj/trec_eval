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


# Known measures supported by pytrec_eval
supported_measures: Set[str] = {
    "official", "all_trec", "set",
    "runid", "num_ret", "num_rel", "num_rel_ret", "num_nonrel_judged_ret",
    "map", "map_cut", "gm_map", "Rprec", "Rprec_mult", "recip_rank",
    "P", "relative_P", "bpref", "gm_bpref", "binG", "G",
    "ndcg", "dcg", "ideal_dcg", "ndcg_cut", "ndcg_rel", "ndcg_p", "Rndcg",
    "recall", "success", "11pt_avg", "iprec_at_recall", "utility", "relstring",
    "set_relative_P", "set_map", "set_F", "infap", "unj", "rbp", "rbp_resid", "yaap",
}

# Measures that print one row per parameter value. pytrec_eval merges the
# parameter lists of repeated requests for one of these, such as P_5 and P.10.
_ROW_PER_VALUE = (
    "P", "recall", "success", "unj", "ndcg_cut", "map_cut", "relative_P",
    "iprec_at_recall", "Rprec_mult",
)
_VALUE_REQUEST = re.compile(
    rf"({'|'.join(_ROW_PER_VALUE)})[_.]([0-9]+(?:\.[0-9]*)?(?:,[0-9]+(?:\.[0-9]*)?)*)"
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
        native_measures: List[str] = []
        positions: Dict[str, int] = {}
        values: Dict[str, Dict[Decimal, str]] = collections.defaultdict(dict)
        for measure in measure_names:
            match = _VALUE_REQUEST.fullmatch(measure)
            if match is None:
                native_measures.append(measure)
                continue
            name = match[1]
            if name not in positions:
                positions[name] = len(native_measures)
                native_measures.append(name)
            for value in match[2].split(","):
                values[name].setdefault(Decimal(value), value)
        for name, index in positions.items():
            listed = values[name]
            native_measures[index] = f"{name}.{','.join(listed[v] for v in sorted(listed))}"

        self._evaluator = Evaluator(
            qrels=qrel,
            measures=native_measures,
            relevance_level=relevance_level,
            judged_docs_only=judged_docs_only_flag,
        )
        self.measures = set(measure_names)

    def evaluate(self, run: Any) -> Dict[str, Dict[str, float]]:
        """Evaluate run and return nested dict matching pytrec_eval output format."""
        try:
            eval_result = self._evaluator.evaluate(run)
        except NoSharedTopicError:
            # pytrec_eval returns {} for a run that shares no topic with the qrels.
            return {}
        return eval_result.to_dict()


__all__ = [
    "RelevanceEvaluator",
    "parse_run",
    "parse_qrel",
    "supported_measures",
    "__version__",
]
