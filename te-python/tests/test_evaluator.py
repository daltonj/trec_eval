import math
import os
import pytest
from pathlib import Path
import trec_eval
from trec_eval import Evaluator, EvalResult, ScoredDoc, Qrel, evaluate

from conftest import One

TEST_DATA_DIR = Path(__file__).resolve().parent.parent.parent / "te-rust" / "tests" / "test-data"
QRELS_FILE = str(TEST_DATA_DIR / "qrels.test")
RESULTS_FILE = str(TEST_DATA_DIR / "results.test")


@pytest.fixture
def sample_qrels_dict():
    return {
        "q1": {
            "d1": 0,
            "d2": 1,
            "d3": 0,
        },
        "q2": {
            "d2": 1,
            "d3": 1,
        },
    }


@pytest.fixture
def sample_run_dict():
    return {
        "q1": {
            "d1": 1.0,
            "d2": 0.0,
            "d3": 1.5,
        },
        "q2": {
            "d1": 1.5,
            "d2": 0.2,
            "d3": 0.5,
        },
    }


def test_evaluator_dict_inputs(sample_qrels_dict, sample_run_dict):
    evaluator = Evaluator(sample_qrels_dict, measures=["map", "ndcg@10", "recip_rank", "P.5"])
    res = evaluator.evaluate(sample_run_dict)

    assert isinstance(res, EvalResult)
    assert len(res) == 2
    assert "q1" in res
    assert "q2" in res

    # Per query checks
    q1 = res["q1"]
    assert "map" in q1
    assert "ndcg_cut_10" in q1
    assert "recip_rank" in q1
    assert "P_5" in q1

    # Aggregates
    agg = res.aggregate()
    assert "map" in agg
    assert "ndcg_cut_10" in agg
    assert "recip_rank" in agg
    assert "P_5" in agg
    assert agg["map"] == pytest.approx((res["q1"]["map"] + res["q2"]["map"]) / 2.0)


def test_evaluator_tuple_and_namedtuple_inputs():
    qrels = [
        Qrel("q1", "d1", 0),
        Qrel("q1", "d2", 1),
        Qrel("q2", "d2", 1),
    ]
    run = [
        ScoredDoc("q1", "d1", 0.5),
        ScoredDoc("q1", "d2", 1.5),
        ScoredDoc("q2", "d2", 0.8),
    ]
    evaluator = Evaluator(qrels, measures=["map", "recip_rank"])
    res = evaluator.evaluate(run)

    assert len(res) == 2
    assert res["q1"]["recip_rank"] == 1.0
    assert res["q1"]["map"] == 1.0


def test_evaluator_file_inputs():
    assert os.path.exists(QRELS_FILE)
    assert os.path.exists(RESULTS_FILE)

    evaluator = Evaluator(QRELS_FILE, measures=["map", "ndcg@10", "P.5,10", "bpref"])
    res = evaluator.evaluate(RESULTS_FILE)

    assert len(res) > 0
    agg = res.aggregate()
    assert "map" in agg
    assert "ndcg_cut_10" in agg
    assert "bpref" in agg
    assert "P_5" in agg
    assert "P_10" in agg


def test_functional_evaluate_one_liner(sample_qrels_dict, sample_run_dict):
    res = evaluate(sample_qrels_dict, sample_run_dict, measures=["map", "recip_rank"])
    assert isinstance(res, EvalResult)
    assert "q1" in res
    assert "map" in res.aggregate()


def test_eval_result_mapping_protocol(sample_qrels_dict, sample_run_dict):
    res = evaluate(sample_qrels_dict, sample_run_dict, measures=["map"])

    # Mapping protocol
    assert list(res) == ["q1", "q2"]
    assert res.keys() == ["q1", "q2"]
    assert len(res.values()) == 2
    assert len(res.items()) == 2

    for qid, scores in res.items():
        assert qid in ["q1", "q2"]
        assert "map" in scores

    # dict conversion
    d = res.to_dict()
    assert isinstance(d, dict)
    assert set(d.keys()) == {"q1", "q2"}

    # KeyError on missing query
    with pytest.raises(KeyError):
        _ = res["nonexistent_query"]


def test_measure_aliases_and_groups(sample_qrels_dict, sample_run_dict):
    evaluator = Evaluator(
        sample_qrels_dict,
        measures=["ndcg@10", "map@5", "P@5,10", "mrr", "recall@10"]
    )
    res = evaluator.evaluate(sample_run_dict)
    agg = res.aggregate()
    assert "ndcg_cut_10" in agg
    assert "map_cut_5" in agg
    assert "P_5" in agg
    assert "P_10" in agg
    assert "recip_rank" in agg
    assert "recall_10" in agg


def test_config_switches(sample_qrels_dict, sample_run_dict):
    # Complete set average with disjoint query in qrels
    qrels_extra = dict(sample_qrels_dict)
    qrels_extra["q3"] = {"d1": 1}

    eval_no_c = Evaluator(qrels_extra, measures=["map"], complete_set_average=False)
    res_no_c = eval_no_c.evaluate(sample_run_dict)
    # 2 queries evaluated in run -> average over 2
    map_no_c = res_no_c.aggregate()["map"]

    eval_c = Evaluator(qrels_extra, measures=["map"], complete_set_average=True)
    res_c = eval_c.evaluate(sample_run_dict)
    # average over all 3 qrels queries (q3 gets 0.0)
    map_c = res_c.aggregate()["map"]

    assert map_c == pytest.approx(map_no_c * 2.0 / 3.0)


def test_all_query_values_adds_the_measures_printed_only_in_the_summary():
    qrels = {"1": {"a": 1, "b": 0}}
    run = {"1": {"b": 2.0, "a": 1.0}}
    measures = ["gm_map", "num_q", "runid", "map"]
    assert Evaluator(qrels, measures).evaluate(run).to_dict() == {"1": {"map": 0.5}}
    assert Evaluator(qrels, measures, all_query_values=True).evaluate(run).to_dict() == {
        "1": {"gm_map": pytest.approx(math.log(0.5)), "num_q": 1, "runid": "trec_eval", "map": 0.5}
    }


def test_numerical_parity_against_known_run():
    # Evaluate official measure set on test data
    evaluator = Evaluator(
        QRELS_FILE, measures=["map", "Rprec", "recip_rank", "bpref", "P.5,10"]
    )
    res = evaluator.evaluate(RESULTS_FILE)
    agg = res.aggregate()

    # Verify non-trivial scores within [0, 1]
    for measure_name in ["map", "Rprec", "recip_rank", "bpref", "P_5", "P_10"]:
        score = agg[measure_name]
        assert 0.0 <= score <= 1.0, f"{measure_name} score {score} outside [0, 1]"


def test_negative_relevance_level_is_refused():
    level = -1
    with pytest.raises(ValueError, match=f"Unusable relevance level '{level}'"):
        Evaluator({"q": {"a": 1}}, relevance_level=level)


def test_run_without_a_qrels_topic_raises():
    qrels = {"q1": {"d1": 1}}
    evaluator = Evaluator(qrels, measures=["map", "num_q"])
    with pytest.raises(ValueError, match="no topic has both results and judgments"):
        evaluator.evaluate({"q3": {"d1": 1.0}})
    with pytest.raises(ValueError, match="no topic has both results and judgments"):
        evaluator.evaluate({})
    with pytest.raises(ValueError, match="no topic has both results and judgments"):
        evaluator.evaluate_arrays(["q3"], ["d1"], [1.0])


def test_complete_set_averages_builtin_and_custom_measures_without_overlap():
    qrels = {"q1": {"d1": 1}}
    run = {"q3": {"d1": 1.0}}
    evaluator = Evaluator(
        qrels, measures=["map", "num_q", One()], complete_set_average=True
    )
    result = evaluator.evaluate(run)
    assert result.per_query() == {"q1": {"map": 0.0, "one": 1.0}}
    assert result.aggregate() == {"num_q": 1, "map": 0.0, "one": 1.0}


def test_integer_cutoff_reads_its_leading_integer_with_a_warning():
    qrels = {"1": {"a": 1, "b": 0}}
    with pytest.warns(UserWarning, match=r"measure 'P\.5x': value '5x' partially read as '5'"):
        evaluator = Evaluator(qrels, measures=["P.5x"])
    # One relevant document in the top 5.
    assert evaluator.evaluate({"1": {"a": 2.0, "b": 1.0}}).aggregate()["P_5"] == 0.2


def test_integer_cutoff_without_a_leading_integer_is_refused():
    with pytest.raises(ValueError, match="invalid integer cutoff 'x'"):
        Evaluator({"1": {"a": 1}}, measures=["P.x"])


def test_first_explicit_parameter_list_wins():
    with pytest.warns(UserWarning, match="measure 'P.5,10': parameters ignored"):
        evaluator = Evaluator({"1": {"a": 1}}, measures=["P.2,5", "P.5,10"])
    assert evaluator.evaluate({"1": {"a": 1.0}}).aggregate() == {"P_2": 0.5, "P_5": 0.2}


@pytest.mark.parametrize("measure", ["set_F.nan", "Rprec_mult.1e400"])
def test_non_finite_parameter_is_refused(measure):
    with pytest.raises(ValueError, match="not a finite number"):
        Evaluator({"1": {"a": 1, "b": 0}}, measures=[measure])


@pytest.mark.parametrize("representation", ["dict", "frame", "tuples"])
def test_in_memory_qrels_refuse_out_of_range_grades(representation, qrels_as):
    for grade in [-128, 128]:
        with pytest.raises(
            ValueError, match=f"relevance level {grade}.*outside -127 to 127"
        ):
            Evaluator(qrels_as(representation, [("q", "d", grade)]), measures=["map"])


def test_in_memory_qrels_accept_grade_boundaries():
    for grade, expected in [(-127, 0.0), (127, 1.0)]:
        result = Evaluator({"q": {"d": grade}}, measures=["map"]).evaluate(
            {"q": {"d": 1.0}}
        )
        assert result.aggregate() == {"map": expected}


def test_file_warnings_reach_python(tmp_path):
    qrels, run = tmp_path / "qrels", tmp_path / "run"
    qrels.write_text("q 0 a 1.5\n")
    run.write_text("q Q0 a 1 1e400x r\n")
    with pytest.warns(UserWarning, match="value '1.5' partially read as '1'"):
        evaluator = Evaluator(qrels, measures=["map"])
    with pytest.warns(UserWarning) as caught:
        result = evaluator.evaluate(run)
    assert [str(w.message) for w in caught] == [
        f"run file '{run}': Line 1: score '1e400x' out of range, read as inf",
        f"run file '{run}': Line 1: score '1e400x' partially read as '1e400'",
    ]
    assert result.aggregate() == {"map": 1.0}


def test_infinite_scores_are_accepted_from_memory():
    evaluator = Evaluator({"q": {"a": 1}}, measures=["P.1"])
    for score, expected in [(math.inf, 0.0), (-math.inf, 1.0)]:
        assert evaluator.evaluate({"q": {"a": 0.0, "x": score}}).aggregate() == {
            "P_1": expected
        }


def test_nan_scores_are_refused_from_both_evaluation_entry_points():
    evaluator = Evaluator({"q": {"a": 1}}, measures=["map"])
    for score in [
        lambda: evaluator.evaluate({"q": {"a": math.nan}}),
        lambda: evaluator.evaluate_arrays(["q"], ["a"], [math.nan]),
    ]:
        with pytest.raises(
            ValueError, match="score of document 'a' for query 'q' is NaN"
        ):
            score()


@pytest.mark.parametrize("measure, name", [("map", "map"), (One(), "one")])
def test_repeated_run_documents_are_checked_only_within_max_docs(measure, name):
    args = ["q", "q"], ["a", "a"], [2.0, 1.0]
    with pytest.raises(ValueError, match="run: duplicate document 'a' for query 'q'"):
        Evaluator({"q": {"a": 1}}, [measure]).evaluate_arrays(*args)
    cut = Evaluator({"q": {"a": 1}}, [measure], max_docs_per_topic=1)
    assert cut.evaluate_arrays(*args).aggregate() == {name: 1.0}


@pytest.mark.parametrize("representation", ["file", "tuples", "frame"])
def test_ingestion_preserves_repeated_judgments(representation, qrels_as):
    qrels = qrels_as(representation, [("q", "a", 0), ("q", "a", 1)])
    evaluator = Evaluator(qrels, measures=["map"])
    with pytest.raises(ValueError, match="qrels: duplicate document 'a' for query 'q'"):
        evaluator.evaluate({"q": {"a": 1.0}})


@pytest.mark.parametrize(
    "gains, message",
    [
        ("1=x", "invalid gain 'x' for relevance level 1"),
        ("1=2,1=5", "relevance level 1 is given more than once"),
    ],
)
def test_unusable_global_gains_are_refused(gains, message):
    with pytest.raises(ValueError, match=f"Invalid global_gains: {message}"):
        Evaluator({"q": {"d": 1}}, measures=["ndcg"], global_gains=gains)


def test_legal_global_gains_are_applied():
    # Ranking gains are 3, 9; ideal gains are 9, 3, discounted by log2(rank + 1).
    evaluator = Evaluator(
        {"t": {"a": 2, "b": 1}}, measures=["ndcg"], global_gains="1=3,2=9"
    )
    assert evaluator.evaluate({"t": {"b": 2.0, "a": 1.0}}).aggregate()[
        "ndcg"
    ] == pytest.approx((3 + 9 / math.log2(3)) / (9 + 3 / math.log2(3)), rel=1e-12)
