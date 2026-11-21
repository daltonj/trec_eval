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


def test_qrels_grades_read_their_leading_integer_with_a_warning(tmp_path):
    qrels = tmp_path / "grades.qrels"
    qrels.write_text("t 0 a 1.5\nt 0 b 0.9\nt 0 c 0\n")
    with pytest.warns(UserWarning) as caught:
        evaluator = Evaluator(str(qrels), measures=["map"])
    messages = [str(w.message) for w in caught]
    assert any("value '1.5' partially read as '1'" in m for m in messages), messages
    assert any("value '0.9' partially read as '0'" in m for m in messages), messages
    # Only a (1.5 read as 1) is relevant, at rank 2.
    run = {"t": {"b": 3.0, "a": 2.0, "c": 1.0}}
    assert evaluator.evaluate(run).aggregate()["map"] == 0.5
