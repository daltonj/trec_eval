import pytest

import trec_eval.compat.pytrec_eval as pytrec_eval
from trec_eval import Evaluator

QRELS = {"q1": {"d1": 1, "d2": 0}, "q2": {"d1": 0, "d2": 1}}
OTHER_TOPIC = {"q3": {"d1": 1.0}}


def test_run_without_a_qrels_topic_raises():
    evaluator = Evaluator(QRELS, measures=["map", "num_q"])
    with pytest.raises(ValueError, match="no topic has both results and judgments"):
        evaluator.evaluate(OTHER_TOPIC)
    with pytest.raises(ValueError, match="no topic has both results and judgments"):
        evaluator.evaluate({})
    with pytest.raises(ValueError, match="no topic has both results and judgments"):
        evaluator.evaluate_arrays(["q3"], ["d1"], [1.0])


def test_complete_set_average_scores_the_qrels_topics():
    evaluator = Evaluator(QRELS, measures=["map"], complete_set_average=True)
    # Both qrels topics are scored, with nothing retrieved: map 0 each.
    assert evaluator.evaluate(OTHER_TOPIC).per_query() == {"q1": {"map": 0.0}, "q2": {"map": 0.0}}


def test_partial_overlap_is_unchanged():
    # q1 ranks d1, its only relevant document, first: map 1.
    result = Evaluator(QRELS, measures=["map", "num_q"]).evaluate({"q1": {"d1": 1.0}, "q3": {"d1": 1.0}})
    assert result.aggregate() == {"map": 1.0, "num_q": 1}


def test_pytrec_eval_compat_returns_no_topics_as_pytrec_eval_does():
    # pytrec_eval 0.5 returns {} for a run that shares no topic with the qrels.
    evaluator = pytrec_eval.RelevanceEvaluator(QRELS, {"map"})
    assert evaluator.evaluate(OTHER_TOPIC) == {}
    assert evaluator.evaluate({}) == {}
    assert evaluator.evaluate({"q1": {"d1": 1.0}, "q3": {"d1": 1.0}}) == {"q1": {"map": 1.0}}


@pytest.mark.parametrize("qrels_kind", ["dict", "file", "iterable", "frame"])
@pytest.mark.parametrize("run_kind", ["dict", "file", "iterable", "frame"])
def test_compat_no_overlap_after_ingestion(tmp_path, qrels_kind, run_kind):
    from test_stage3 import MockDataFrame

    qrels_file = tmp_path / "topic.qrels"
    qrels_file.write_text("q1 0 d 1\n")
    run_file = tmp_path / "topic.run"
    run_file.write_text("q2 Q0 d 1 1 run\n")
    qrels = {
        "dict": {"q1": {"d": 1}},
        "file": qrels_file,
        "iterable": iter([("q1", "d", 1)]),
        "frame": MockDataFrame({"qid": ["q1"], "docno": ["d"], "rel": [1]}),
    }
    runs = {
        "dict": {"q2": {"d": 1.0}},
        "file": run_file,
        "iterable": iter([("q2", "d", 1.0)]),
        "frame": MockDataFrame({"qid": ["q2"], "docno": ["d"], "score": [1.0]}),
    }
    evaluator = pytrec_eval.RelevanceEvaluator(qrels[qrels_kind], {"map"})
    assert evaluator.evaluate(runs[run_kind]) == {}
    assert evaluator.evaluate({"q1": {"d": 1.0}}) == {"q1": {"map": 1.0}}


def test_compat_keeps_nan_score_errors():
    evaluator = pytrec_eval.RelevanceEvaluator(QRELS, {"map"})
    with pytest.raises(ValueError, match="NaN"):
        evaluator.evaluate({"q1": {"d1": float("nan")}})


def test_compat_keeps_other_evaluation_errors(tmp_path):
    qrels = tmp_path / "duplicate.qrels"
    qrels.write_text("q1 0 d 0\nq1 0 d 1\n")
    evaluator = pytrec_eval.RelevanceEvaluator(qrels, {"map"})
    with pytest.raises(ValueError, match="qrels: duplicate document"):
        evaluator.evaluate({"q1": {"d": 1.0}})
