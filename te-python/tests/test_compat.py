import os
from pathlib import Path
import pytest
import trec_eval.compat.pytrec_eval as pytrec_eval

TEST_DATA_DIR = Path(__file__).resolve().parent.parent.parent / "te-rust" / "tests" / "test-data"
QRELS_FILE = str(TEST_DATA_DIR / "qrels.test")
RESULTS_FILE = str(TEST_DATA_DIR / "results.test")


def test_pytrec_eval_drop_in_relevance_evaluator():
    qrels = {
        "q1": {"d1": 0, "d2": 1, "d3": 0},
        "q2": {"d2": 1, "d3": 1},
    }
    run = {
        "q1": {"d1": 1.0, "d2": 0.0, "d3": 1.5},
        "q2": {"d1": 1.5, "d2": 0.2, "d3": 0.5},
    }

    evaluator = pytrec_eval.RelevanceEvaluator(qrels, {"map", "ndcg"})
    res = evaluator.evaluate(run)

    # Returns exact nested dict
    assert isinstance(res, dict)
    assert isinstance(res["q1"], dict)
    assert "map" in res["q1"]
    assert "ndcg" in res["q1"]
    assert res["q1"]["map"] == pytest.approx(1.0 / 3.0)
    assert res["q1"]["ndcg"] == pytest.approx(0.5)


@pytest.mark.parametrize(
    "measures, expected",
    [
        (["P_5", "P_10"], {"P_5": 0.2, "P_10": 0.1}),
        (["P_10", "P_5"], {"P_5": 0.2, "P_10": 0.1}),
        ({"P_5", "P_10"}, {"P_5": 0.2, "P_10": 0.1}),
        (["P_05", "P_5"], {"P_5": 0.2}),
        (["ndcg_cut_5", "ndcg_cut_10"], {"ndcg_cut_5": 1.0, "ndcg_cut_10": 1.0}),
        (["ndcg_cut_10", "ndcg_cut_5"], {"ndcg_cut_5": 1.0, "ndcg_cut_10": 1.0}),
        ({"ndcg_cut_5", "ndcg_cut_10"}, {"ndcg_cut_5": 1.0, "ndcg_cut_10": 1.0}),
        (["P.5", "P.10"], {"P_5": 0.2, "P_10": 0.1}),
        (["P.5", "P_10"], {"P_5": 0.2, "P_10": 0.1}),
        (["Rprec_mult.0.5", "Rprec_mult_2"], {"Rprec_mult_0.50": 1.0, "Rprec_mult_2.00": 0.5}),
        (["Rprec_mult.1.", "Rprec_mult.2."], {"Rprec_mult_1.00": 1.0, "Rprec_mult_2.00": 0.5}),
        (["iprec_at_recall.0.5", "iprec_at_recall_1.0"], {"iprec_at_recall_0.50": 1.0, "iprec_at_recall_1.00": 1.0}),
    ],
)
def test_pytrec_eval_retains_requested_cutoffs(measures, expected):
    evaluator = pytrec_eval.RelevanceEvaluator({"1": {"a": 1}}, measures)
    assert evaluator.evaluate({"1": {"a": 1.0}}) == {"1": expected}


@pytest.mark.parametrize(
    "measures, expected",
    [
        (["map@1"], {"map_cut_1": 1.0}),
        (["ndcg@1"], {"ndcg_cut_1": 1.0}),
        (["utility.1,0,0,0"], {"utility_1,0,0,0": 1.0}),
    ],
)
def test_pytrec_eval_passes_other_parameters_unchanged(measures, expected):
    evaluator = pytrec_eval.RelevanceEvaluator({"1": {"a": 1}}, measures)
    assert evaluator.evaluate({"1": {"a": 1.0}}) == {"1": expected}


def test_pytrec_eval_parse_helpers():
    assert os.path.exists(QRELS_FILE)
    assert os.path.exists(RESULTS_FILE)

    qrels_dict = pytrec_eval.parse_qrel(QRELS_FILE)
    run_dict = pytrec_eval.parse_run(RESULTS_FILE)

    assert isinstance(qrels_dict, dict)
    assert isinstance(run_dict, dict)
    assert len(qrels_dict) > 0
    assert len(run_dict) > 0

    evaluator = pytrec_eval.RelevanceEvaluator(qrels_dict, {"map", "recip_rank"})
    res = evaluator.evaluate(run_dict)
    assert len(res) > 0


def test_pytrec_eval_supported_measures():
    assert "map" in pytrec_eval.supported_measures
    assert "ndcg" in pytrec_eval.supported_measures
    assert "recip_rank" in pytrec_eval.supported_measures
    assert "official" in pytrec_eval.supported_measures



@pytest.mark.parametrize("measure", ["dcg", "ideal_dcg"])
def test_supported_dcg_measures_can_be_evaluated(measure):
    assert measure in pytrec_eval.supported_measures
    evaluator = pytrec_eval.RelevanceEvaluator({"q": {"d": 1}}, {measure})
    assert evaluator.evaluate({"q": {"d": 1.0}}) == {"q": {measure: 1.0}}
