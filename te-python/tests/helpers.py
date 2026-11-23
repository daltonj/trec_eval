"""Helpers shared by the evaluator tests."""

from trec_eval import Measure, QueryState


class MockDataFrame:
    """Mock DataFrame implementing column access and .columns attribute."""
    def __init__(self, data: dict):
        self.data = data
        self.columns = list(data.keys())

    def __getitem__(self, col):
        return self.data[col]


class One(Measure):
    def __init__(self):
        super().__init__(name="one")

    def calc_query(self, query: QueryState) -> float:
        return 1.0
