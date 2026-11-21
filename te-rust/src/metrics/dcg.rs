//! `dcg` and `ideal_dcg`, the numerator and the denominator of `ndcg`, which
//! `m_ndcg.c` registers beside it.

use crate::metrics::common::{topic_gains, GainsConfig, MeasureParseError};
use crate::metrics::invariants::{self, Invariant};
use crate::metrics::ndcg::{ideal_dcg, ranking_dcg};
use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};

/// Which of the two sums of `ndcg` a measure reports.
#[derive(Clone, Copy)]
enum Sum {
    Ranking,
    Ideal,
}

impl Sum {
    fn name(self) -> &'static str {
        match self {
            Sum::Ranking => "dcg",
            Sum::Ideal => "ideal_dcg",
        }
    }
}

/// The ideal ranking's DCG depends on the judgments alone, so an empty ranking does
/// not make it 0.
const IDEAL_DCG_INVARIANTS: &[Invariant] = &[
    Invariant::Finite,
    Invariant::NonNegative,
    Invariant::ZeroRelevanceIsZero,
];

/// `dcg` or `ideal_dcg`.
pub struct DcgMeasure {
    sum: Sum,
    gains_config: GainsConfig,
    sub_metrics: Vec<String>,
}

impl DcgMeasure {
    /// `dcg`, the DCG of the ranking.
    pub fn new(params_str: &str) -> Result<Self, MeasureParseError> {
        Self::build(Sum::Ranking, params_str)
    }

    /// `ideal_dcg`, the DCG of the ideal ranking.
    pub fn ideal(params_str: &str) -> Result<Self, MeasureParseError> {
        Self::build(Sum::Ideal, params_str)
    }

    fn build(sum: Sum, params_str: &str) -> Result<Self, MeasureParseError> {
        let name = if params_str.is_empty() {
            sum.name().to_string()
        } else {
            format!("{}_{}", sum.name(), params_str)
        };
        Ok(Self {
            sum,
            gains_config: GainsConfig::parse(params_str)?,
            sub_metrics: vec![name],
        })
    }
}

impl Measure for DcgMeasure {
    fn name(&self) -> &'static str {
        self.sum.name()
    }

    fn short_description(&self) -> &'static str {
        match self.sum {
            Sum::Ranking => "Discounted Cumulative Gain",
            Sum::Ideal => "Ideal Discounted Cumulative Gain",
        }
    }

    fn explanation(&self) -> &'static str {
        match self.sum {
            Sum::Ranking => {
                "Discounted Cumulative Gain. Sums gain / log2(rank + 1) over the entire retrieved ranking; the numerator of ndcg."
            }
            Sum::Ideal => {
                "Ideal Discounted Cumulative Gain. The DCG of the judged documents with a positive gain, in descending order of gain; the denominator of ndcg."
            }
        }
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn sub_metrics(&self) -> Vec<String> {
        self.sub_metrics.clone()
    }

    /// Invariants are checked with the default gains, as for `ndcg`; a gain map with
    /// negative gains can make `dcg` negative.
    fn invariants(&self) -> &'static [Invariant] {
        match self.sum {
            Sum::Ranking => invariants::STANDARD,
            Sum::Ideal => IDEAL_DCG_INVARIANTS,
        }
    }

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };
        let gains = topic_gains(&self.gains_config, config, q_state);
        let value = match self.sum {
            Sum::Ranking => ranking_dcg(&gains, q_state),
            Sum::Ideal => ideal_dcg(&gains),
        };
        vec![MetricValue::Float(value)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;
    use crate::metrics::ndcg::NdcgMeasure;

    #[test]
    fn dcg_and_ideal_dcg_are_the_two_sums_of_ndcg() {
        // The ranking of test_ndcg_standard in ndcg.rs: dcg = 1.5 and ideal_dcg = 1 + 1/log2(3).
        let config = EvalConfig::default();
        let state = EvalState::Standard(make_mock_state(vec![1, 0, 1], 2));
        let value = |values: Vec<MetricValue>| match values[..] {
            [MetricValue::Float(v)] => v,
            _ => panic!("Expected one float value"),
        };
        let dcg = value(DcgMeasure::new("").unwrap().calc(&config, &state));
        let ideal = value(DcgMeasure::ideal("").unwrap().calc(&config, &state));
        let ndcg = value(NdcgMeasure::new("").unwrap().calc(&config, &state));
        assert!((dcg - 1.5).abs() < 1e-12);
        assert!((ideal - (1.0 + 1.0 / 3f64.log2())).abs() < 1e-12);
        assert!((ndcg - dcg / ideal).abs() < 1e-12);
    }
}
