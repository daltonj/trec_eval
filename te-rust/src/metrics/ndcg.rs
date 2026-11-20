use crate::eval::QueryEvalState;
use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use crate::metrics::common::{Gains, GainsConfig, MeasureParseError};

pub struct NdcgMeasure {
    gains_config: GainsConfig,
    sub_metrics: Vec<String>,
}

impl NdcgMeasure {
    pub fn new(params_str: &str) -> Result<Self, MeasureParseError> {
        let name = if params_str.is_empty() {
            "ndcg".to_string()
        } else {
            format!("ndcg_{}", params_str)
        };
        Ok(Self {
            gains_config: GainsConfig::parse(params_str)?,
            sub_metrics: vec![name],
        })
    }
}

/// The gains of a topic's levels. A measure's own gain map comes first, then
/// `--global-gains`, then the default gains.
pub(crate) fn topic_gains(
    own_gains: &GainsConfig,
    config: &EvalConfig,
    q_state: &QueryEvalState,
) -> Gains {
    let gains_config = if !own_gains.custom_gains.is_empty() {
        own_gains
    } else if let Some(ref gg) = config.global_gains {
        gg
    } else {
        own_gains
    };
    Gains::setup(gains_config, &q_state.rel_levels)
}

/// Document i, counting from 0, has rank i + 1 and discount log2(i + 2).
fn discount(i: usize) -> f64 {
    ((i + 2) as f64).log2()
}

/// The DCG of the ranking, the numerator of `ndcg`: every retrieved document, whatever
/// its gain.
pub(crate) fn ranking_dcg(gains: &Gains, q_state: &QueryEvalState) -> f64 {
    let mut dcg = 0.0;
    let ranking = q_state.results_rel_list.iter().take(q_state.num_ret);
    for (i, &rel) in ranking.enumerate() {
        let gain = gains.get_gain(rel);
        if gain != 0.0 {
            dcg += gain / discount(i);
        }
    }
    dcg
}

/// The DCG of the ideal ranking, the denominator of `ndcg`: the judged documents with a
/// positive gain, in descending order of gain.
pub(crate) fn ideal_dcg(gains: &Gains) -> f64 {
    // `Gains::setup` sorts the levels by increasing gain.
    let mut dcg = 0.0;
    let mut i = 0;
    let positive = gains
        .rel_gains
        .iter()
        .rev()
        .take_while(|level| level.gain > 0.0);
    for level in positive {
        for _ in 0..level.num_at_level {
            dcg += level.gain / discount(i);
            i += 1;
        }
    }
    dcg
}

impl Measure for NdcgMeasure {
    fn name(&self) -> &'static str {
        "ndcg"
    }

    fn short_description(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain"
    }

    fn explanation(&self) -> &'static str {
        "Normalized Discounted Cumulative Gain. Computes NDCG over the entire retrieved ranking and entire ideal ranking."
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

    fn initial_values(&self) -> Vec<MetricValue> {
        vec![MetricValue::Float(0.0)]
    }

    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue> {
        let q_state = match state.get_standard() {
            Some(q) => q,
            None => return self.initial_values(),
        };

        let gains = topic_gains(&self.gains_config, config, q_state);
        let ideal = ideal_dcg(&gains);
        let score = if ideal > 0.0 { ranking_dcg(&gains, q_state) / ideal } else { 0.0 };
        vec![MetricValue::Float(score)]
    }

}

impl Default for NdcgMeasure {
    fn default() -> Self {
        Self::new("").expect("an empty gain map is valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_ndcg_standard() {
        let measure = NdcgMeasure::new("").unwrap();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1], num_rel = 2.
        // dcg = 1.0 / log2(2) + 0.0 + 1.0 / log2(4) = 1.0 + 0.5 = 1.5.
        // idcg = 1.0 / log2(2) + 1.0 / log2(3) = 1.0 + 1.0 / 1.58496 = 1.63092975
        // ndcg = 1.5 / 1.63092975 = 0.91972079
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual.len(), 1);
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.919720791448).abs() < 1e-6);
        } else {
            panic!("Expected float value");
        }
    }
}
