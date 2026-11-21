import pytest
from trec_eval import Evaluator

QRELS = {"t": {"a": 2, "b": 0, "c": 1, "d": 3}}
RUN = {"t": {"d": 5.0, "x": 4.0, "a": 3.0, "b": 2.0, "c": 1.0}}


@pytest.mark.parametrize(
    "measure, message",
    [
        ("ndcg.1=x", "invalid gain 'x' for relevance level 1"),
        ("ndcg.x=1", "invalid relevance level 'x'"),
        ("ndcg.5,5", "expected key=value, got '5'"),
        ("ndcg.1=2,1=5", "relevance level 1 is given more than once"),
    ],
)
def test_unusable_gain_map_is_refused(measure, message):
    with pytest.raises(ValueError, match=message):
        Evaluator(QRELS, measures=[measure])


@pytest.mark.parametrize(
    "gains, message",
    [
        ("1=x", "invalid gain 'x' for relevance level 1"),
        ("x=1", "invalid relevance level 'x'"),
        ("1=2,1=5", "relevance level 1 is given more than once"),
    ],
)
def test_unusable_global_gains_are_refused(gains, message):
    with pytest.raises(ValueError, match=f"Invalid global_gains: {message}"):
        Evaluator(QRELS, measures=["ndcg"], global_gains=gains)


def test_legal_global_gains_are_applied():
    # As `te-rust --global-gains 1=3,2=9 -m ndcg` and trec_eval's `ndcg.1=3,2=9`: 0.6988.
    evaluator = Evaluator(QRELS, measures=["ndcg"], global_gains="1=3,2=9")
    assert evaluator.evaluate(RUN).aggregate()["ndcg"] == pytest.approx(0.6988385132278441, rel=1e-12)
