"""Helpers shared by the evaluator tests."""


class MockDataFrame:
    """Mock DataFrame implementing column access and .columns attribute."""
    def __init__(self, data: dict):
        self.data = data
        self.columns = list(data.keys())

    def __getitem__(self, col):
        return self.data[col]
