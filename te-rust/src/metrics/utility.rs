use crate::metrics::{EvalConfig, EvalState, EvaluationType, Measure, MetricValue, ValueFormat};
use crate::metrics::common::{check_finite, MeasureParseError};

pub struct UtilityMeasure {
    params: Vec<f64>,
    sub_metrics: Vec<String>,
}

impl UtilityMeasure {
    /// Refuses anything other than four finite coefficients.
    pub fn new(params: Vec<f64>, params_str: &str) -> Result<Self, MeasureParseError> {
        if params.len() != 4 {
            return Err(MeasureParseError::new(
                "utility requires exactly 4 coefficients",
            ));
        }
        for &value in &params {
            check_finite(value)?;
        }
        let name = if params_str.is_empty() {
            "utility".to_string()
        } else {
            format!("utility_{}", params_str)
        };
        Ok(Self {
            params,
            sub_metrics: vec![name],
        })
    }
}

impl Measure for UtilityMeasure {
    fn name(&self) -> &'static str {
        "utility"
    }

    fn short_description(&self) -> &'static str {
        "Set utility measure"
    }

    fn explanation(&self) -> &'static str {
        "Utility based on a contingency table of retrieved vs relevant docs. Computed as p1*a + p2*b + p3*c + p4*d, where a is retrieved-relevant, b is retrieved-nonrelevant, c is nonretrieved-relevant, d is nonretrieved-nonrelevant, and p1-p4 are weights (default: 1.0, -1.0, 0.0, 0.0)."
    }

    fn format(&self) -> ValueFormat {
        ValueFormat::Float
    }

    fn eval_type(&self) -> EvaluationType {
        EvaluationType::Standard
    }

    fn invariants(&self) -> &'static [crate::metrics::invariants::Invariant] {
        crate::metrics::invariants::SIGNED
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

        // Signed cells allow a collection size smaller than the documents counted.
        // num_rel_ret counts judged documents only, as trec_eval's does: at a negative -l,
        // unjudged documents (RELVALUE_NONPOOL, RELVALUE_UNJUDGED) pass the level too.
        let num_ret = q_state.results_rel_list.len() as i64;
        let num_rel_ret = q_state.num_rel_ret as i64;
        let num_rel = q_state.num_rel as i64;

        let a = num_rel_ret as f64;
        let b = (num_ret - num_rel_ret) as f64;
        let c = (num_rel - num_rel_ret) as f64;

        // Subtract counts of at least 0 from num_docs_in_coll, so that no step can wrap.
        let num_docs_in_coll = i64::try_from(config.num_docs_in_coll).unwrap_or(i64::MAX);
        let d = (num_docs_in_coll - num_ret - (num_rel - num_rel_ret)) as f64;

        let score = self.params[0] * a +
                    self.params[1] * b +
                    self.params[2] * c +
                    self.params[3] * d;

        vec![MetricValue::Float(score)]
    }
}

impl Default for UtilityMeasure {
    fn default() -> Self {
        Self::new(vec![1.0, -1.0, 0.0, 0.0], "").expect("default coefficients are valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::common::make_mock_state;

    #[test]
    fn test_utility_standard() {
        let measure = UtilityMeasure::default();
        let config = EvalConfig::default();
        // retrieved: [1, 0, 1] (2 relevant, 1 non-relevant), num_rel = 2.
        // default params: [1.0, -1.0, 0.0, 0.0]
        // score = 1.0 * 2 + (-1.0) * 1 = 1.0
        let state = make_mock_state(vec![1, 0, 1], 2);
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(1.0)]);
    }

    #[test]
    fn test_utility_zero_relevance() {
        let measure = UtilityMeasure::default();
        let config = EvalConfig::default();
        let state = make_mock_state(vec![0, 0], 0);
        // a = 0, b = 2, c = 0, d = num_docs_in_coll + 0 - 2 - 0 = num_docs_in_coll - 2.
        // score = 1.0 * 0 + (-1.0) * 2 = -2.0
        let actual = measure.calc(&config, &EvalState::Standard(state));
        assert_eq!(actual, vec![MetricValue::Float(-2.0)]);
    }

    #[test]
    fn utility_counts_only_judged_documents_as_relevant_retrieved() {
        // a = 1, b = 2, c = 1, d = 0 - 3 - (2 - 1) = -4.
        let measure = UtilityMeasure::new(vec![1000.0, 100.0, 10.0, 1.0], "").unwrap();
        let config = EvalConfig {
            relevance_level: -1,
            ..Default::default()
        };
        let mut query = make_mock_state(vec![1, -1, -1], 2);
        query.rel_levels = vec![1, 1];
        query.num_nonpool = 2;
        let state = EvalState::Standard(query);
        assert_eq!(measure.calc(&config, &state), [MetricValue::Float(1206.0)]);
    }
}
