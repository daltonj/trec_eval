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


@pytest.fixture
def qrels_as(tmp_path):
    def convert(kind, rows):
        if kind == "dict":
            grouped = {}
            for qid, docno, grade in rows:
                grouped.setdefault(qid, {})[docno] = grade
            return grouped
        if kind == "frame":
            return MockDataFrame(dict(zip(("qid", "docno", "rel"), zip(*rows))))
        if kind == "tuples":
            return iter(rows)
        if kind == "file":
            path = tmp_path / "input.qrels"
            path.write_text(
                "".join(f"{qid} 0 {docno} {grade}\n" for qid, docno, grade in rows)
            )
            return path
        raise ValueError(kind)

    return convert


class One(Measure):
    def __init__(self):
        super().__init__(name="one")

    def calc_query(self, query: QueryState) -> float:
        return 1.0
