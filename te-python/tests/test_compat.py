import math
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
    assert res["q2"]["map"] == pytest.approx(7.0 / 12.0)
    assert res["q2"]["ndcg"] == pytest.approx(0.6934264036172708)


@pytest.mark.parametrize(
    "flags, expected",
    [
        ({}, {"P_1": 0.0, "num_ret": 3}),
        ({"judged_docs_only_flag": True}, {"P_1": 1.0, "num_ret": 2}),
    ],
)
def test_pytrec_eval_keeps_unjudged_documents_by_default(flags, expected):
    evaluator = pytrec_eval.RelevanceEvaluator(
        {"1": {"d1": 1, "d2": 0}}, {"P_1", "num_ret"}, **flags
    )
    assert evaluator.evaluate({"1": {"u": 3.0, "d1": 2.0, "d2": 1.0}}) == {
        "1": expected
    }


def test_pytrec_eval_gives_every_measure_per_topic_as_a_float():
    evaluator = pytrec_eval.RelevanceEvaluator(
        {"1": {"d1": 1, "d2": 0}},
        {"gm_map", "gm_bpref", "num_q", "runid", "relstring", "num_ret", "map"},
    )
    res = evaluator.evaluate({"1": {"u": 3.0, "d1": 2.0, "d2": 1.0}})
    assert res == {
        "1": {
            "gm_map": pytest.approx(math.log(0.5)),
            "gm_bpref": 0.0,
            "num_q": 1.0,
            "runid": 0.0,
            "relstring": 0.0,
            "num_ret": 3.0,
            "map": 0.5,
        }
    }
    assert all(type(value) is float for value in res["1"].values())


@pytest.mark.parametrize(
    "measures, expected",
    [
        (["P_10", "P.5", "P_05"], {"P_5": 0.2, "P_10": 0.1}),
        (["ndcg_cut_10", "ndcg_cut_5"], {"ndcg_cut_5": 1.0, "ndcg_cut_10": 1.0}),
        (
            ["Rprec_mult.1.", "Rprec_mult_2"],
            {"Rprec_mult_1.00": 1.0, "Rprec_mult_2.00": 0.5},
        ),
        (
            ["iprec_at_recall.0.5", "iprec_at_recall_1.0"],
            {"iprec_at_recall_0.50": 1.0, "iprec_at_recall_1.00": 1.0},
        ),
    ],
)
def test_pytrec_eval_retains_requested_cutoffs(measures, expected):
    evaluator = pytrec_eval.RelevanceEvaluator({"1": {"a": 1}}, measures)
    assert evaluator.evaluate({"1": {"a": 1.0}}) == {"1": expected}


@pytest.mark.parametrize(
    "measures, expected",
    [
        # R = 2; the wrapper stops at 1 in "1e-1", so cutoffs 2 and 4 each hold one relevant doc.
        (
            ["Rprec_mult.1e-1", "Rprec_mult.2"],
            {"Rprec_mult_1.00": 0.5, "Rprec_mult_2.00": 0.25},
        ),
        # The leading list ends at 5x, dropping ,10, and merges the later cutoff 20.
        (["P.5x,10", "P_20"], {"P_5": 0.2, "P_20": 0.05}),
        # Digits must follow a decimal point, so the request ends at 5 and drops ,10.
        (["P.5.,10"], {"P_5": 0.2}),
    ],
)
def test_pytrec_eval_reads_leading_parameter_values(measures, expected):
    evaluator = pytrec_eval.RelevanceEvaluator({"q": {"a": 1, "b": 1}}, measures)
    assert evaluator.evaluate({"q": {"x": 2.0, "a": 1.0}}) == {"q": expected}


@pytest.mark.parametrize(
    "measures, expected",
    [
        (["map@1"], {"map_cut_1": 1.0}),
        (["utility.1,0,0,0"], {"utility_1,0,0,0": 1.0}),
    ],
)
def test_pytrec_eval_passes_other_parameters_unchanged(measures, expected):
    evaluator = pytrec_eval.RelevanceEvaluator({"1": {"a": 1}}, measures)
    assert evaluator.evaluate({"1": {"a": 1.0}}) == {"1": expected}


def test_pytrec_eval_passes_repeated_coefficient_lists_unchanged():
    with pytest.warns(UserWarning, match="parameters ignored"):
        evaluator = pytrec_eval.RelevanceEvaluator(
            {"1": {"a": 1}},
            ["utility.1,0,0,0", "utility.0,1,0,0"],
        )
    assert evaluator.evaluate({"1": {"a": 1.0}}) == {
        "1": {"utility_1,0,0,0": 1.0}
    }


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


def test_compat_scores_only_topics_shared_with_qrels():
    qrels = {"q1": {"d": 1}}
    run = {"q2": {"d": 1.0}}
    evaluator = pytrec_eval.RelevanceEvaluator(qrels, "map")
    assert evaluator.evaluate(run) == {}
    assert evaluator.evaluate({}) == {}
    assert evaluator.evaluate({"q1": {"d": 1.0}, "q2": {"d": 1.0}}) == {
        "q1": {"map": 1.0}
    }


def test_compat_keeps_other_evaluation_errors(tmp_path):
    qrels = tmp_path / "duplicate.qrels"
    qrels.write_text("q1 0 d 0\nq1 0 d 1\n")
    evaluator = pytrec_eval.RelevanceEvaluator(qrels, "map")
    with pytest.raises(ValueError, match="qrels: duplicate document"):
        evaluator.evaluate({"q1": {"d": 1.0}})
