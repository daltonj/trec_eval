from trec_eval import Evaluator


def test_first_explicit_parameter_list_wins():
    evaluator = Evaluator({"1": {"a": 1}}, measures=["P.2,5", "P.5,10"])
    assert evaluator.evaluate({"1": {"a": 1.0}}).aggregate() == {"P_2": 0.5, "P_5": 0.2}
