import pytest
from trec_eval import Evaluator


@pytest.mark.parametrize("grade", [-128, 128, -(2**63), 2**63 - 1])
@pytest.mark.parametrize("representation", ["dict", "frame", "tuples", "objects"])
def test_in_memory_qrels_refuse_out_of_range_grades(grade, representation, qrels_as):
    qrels = qrels_as(representation, [("q", "d", grade)])
    with pytest.raises(
        ValueError, match=f"relevance level {grade} for document 'd' in query 'q' is outside -127 to 127"
    ):
        Evaluator(qrels, measures=["map"])


@pytest.mark.parametrize("grade", [-127, -1, 0, 127])
@pytest.mark.parametrize("representation", ["dict", "frame", "tuples", "objects"])
def test_in_memory_qrels_accept_grade_boundaries(grade, representation, qrels_as):
    qrels = qrels_as(representation, [("q", "d", grade)])
    evaluator = Evaluator(qrels, measures=["map"])
    assert evaluator.evaluate({"q": {"d": 1.0}}).aggregate()["map"] == (
        1.0 if grade > 0 else 0.0
    )
