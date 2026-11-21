from functools import partial
from types import SimpleNamespace

import pytest
from trec_eval import Measure, QueryState


class MockDataFrame:
    """Mock DataFrame implementing column access and .columns attribute."""
    def __init__(self, data: dict):
        self.data = data
        self.columns = list(data.keys())

    def __getitem__(self, col):
        return self.data[col]


@pytest.fixture
def dataframe():
    """Builds a MockDataFrame from a dict of columns."""
    return MockDataFrame


def _rows_as(kind, rows, *, path, qrels):
    if kind == "dict":
        result = {}
        for qid, docno, value in rows:
            result.setdefault(qid, {})[docno] = value
        return result
    if kind == "frame":
        columns = ("qid", "docno", "rel" if qrels else "score")
        return MockDataFrame(dict(zip(columns, zip(*rows))))
    if kind in ("tuples", "iterable"):
        return iter(rows)
    if kind == "objects":
        field = "relevance" if qrels else "score"
        return iter(
            SimpleNamespace(query_id=qid, doc_id=docno, **{field: value})
            for qid, docno, value in rows
        )
    if kind == "arrays":
        return tuple(zip(*rows))
    if kind == "file":
        path.write_text(
            "".join(
                f"{qid} 0 {docno} {value}\n"
                if qrels
                else f"{qid} Q0 {docno} 1 {value} run\n"
                for qid, docno, value in rows
            )
        )
        return path
    raise ValueError(f"Unknown input kind: {kind}")


@pytest.fixture
def qrels_as(tmp_path):
    return partial(_rows_as, path=tmp_path / "input.qrels", qrels=True)


@pytest.fixture
def run_as(tmp_path):
    return partial(_rows_as, path=tmp_path / "input.run", qrels=False)


class One(Measure):
    def __init__(self):
        super().__init__(name="one")

    def calc_query(self, query: QueryState) -> float:
        return 1.0
