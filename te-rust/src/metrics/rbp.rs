use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use crate::metrics::common::{parse_rbp_params, Gains, GainsConfig, MeasureParseError};

pub struct RbpMeasure {
    p: f64,
    gains_config: GainsConfig,
    _params_str: String,
    sub_metrics: Vec<String>,
}

impl RbpMeasure {
    /// Reads `params_str` as `p=<persistence>,<rel>=<gain>,...`: `p`, in [0, 1) and 0.9 by
    /// default, and a gain map.
    pub fn new(params_str: &str) -> Result<Self, MeasureParseError> {
        let (p, gains_config) = parse_rbp_params(params_str)?;
        let name = if params_str.is_empty() {
            "rbp".to_string()
        } else {
            format!("rbp_{}", params_str)
        };
        Ok(Self {
            p,
            gains_config,
            _params_str: params_str.to_string(),
            sub_metrics: vec![name],
        })
    }
}

impl Measure for RbpMeasure {
    fn name(&self) -> &'static str {
        "rbp"
    }

    fn short_description(&self) -> &'static str {
        "Rank-Biased Precision"
    }

    fn explanation(&self) -> &'static str {
        "Rank-Biased Precision. RBP measures the rate at which utility is gained by a user working at a given degree of persistence p. By default, p = 0.9. Higher values of p simulate a more persistent user who reads deeper into the ranking list."
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

        let mut gains = Gains::setup(self.gains_config.or_global(config), &q_state.rel_levels);

        // Find min and max gains to normalize if needed
        let mut min_gain = 0.0;
        let mut max_gain = 0.0;
        for g in &gains.rel_gains {
            if g.gain < min_gain {
                min_gain = g.gain;
            }
            if g.gain > max_gain {
                max_gain = g.gain;
            }
        }

        // Normalize the gain table; a level the table lacks keeps gain 0.
        if min_gain < 0.0 || max_gain > 1.0 {
            let range = max_gain - min_gain;
            for g in &mut gains.rel_gains {
                g.gain = (g.gain - min_gain) / range;
            }
        }

        let mut sum = 0.0;
        let mut cur_p = 1.0;

        for &rel in &q_state.results_rel_list {
            let gain = gains.get_gain(rel);
            if gain != 0.0 {
                sum += gain * cur_p;
            }
            cur_p *= self.p;
        }

        let val = (1.0 - self.p) * sum;
        vec![MetricValue::Float(val)]
    }
}

impl Default for RbpMeasure {
    fn default() -> Self {
        Self::new("").expect("the default parameters are valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_rbp_standard() {
        let measure = RbpMeasure::new("").unwrap();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1]. relevance level = 1.
        // gains at rel levels: rel=0 -> gain=0, rel=1 -> gain=1.
        // normalized gains: max=1, min=0 (standard, no change).
        // rank 1: rel=1 -> gain=1. sum += 1 * p^0 = 1.0
        // rank 2: rel=0 -> gain=0. sum += 0
        // rank 3: rel=1 -> gain=1. sum += 1 * p^2 = 0.81
        // total sum = 1.81.
        // result = (1 - 0.9) * 1.81 = 0.1 * 1.81 = 0.181
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        if let MetricValue::Float(v) = actual[0] {
            assert!((v - 0.181).abs() < 1e-9);
        } else {
            panic!("Expected float value");
        }
    }

    #[test]
    fn test_rbp_custom_p() {
        let measure = RbpMeasure::new("p=0.5").unwrap();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1]. R=2.
        // sum = 1 * 0.5^0 + 0 + 1 * 0.5^2 = 1.0 + 0.25 = 1.25.
        // result = (1 - 0.5) * 1.25 = 0.625
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(0.625)]);
    }
}
