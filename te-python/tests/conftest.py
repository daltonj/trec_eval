import pytest
from helpers import MockDataFrame


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
