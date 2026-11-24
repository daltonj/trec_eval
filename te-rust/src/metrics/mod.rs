pub mod common;
pub mod runid;
pub mod num_q;
pub mod num_ret;
pub mod num_rel;
pub mod num_rel_ret;
pub mod map;
pub mod rprec;
pub mod recip_rank;
pub mod bpref;
pub mod precision;
pub mod ndcg_cut;
pub mod dcg;
pub mod ndcg;
pub mod recall;
pub mod success;
pub mod avg_11pt;
pub mod utility;
pub mod relstring;
pub mod set_p;
pub mod set_recall;
pub mod set_relative_p;
pub mod set_map;
pub mod set_f;
pub mod g;
pub mod map_cut;
pub mod relative_p;
pub mod rprec_mult;
pub mod iprec_at_recall;
pub mod gm_map;
pub mod gm_bpref;
pub mod infap;
pub mod unj;
pub mod num_nonrel_judged_ret;
pub mod rbp;
pub mod rbp_resid;
pub mod yaap;
pub mod bin_g;
pub mod ndcg_rel;
pub mod rndcg;
pub mod ndcg_p;
pub mod map_avgjg;
pub mod precision_avgjg;
pub mod rprec_mult_avgjg;
pub mod registry;
pub mod invariants;


use crate::eval::{EvaluationError, QueryEvalState};
use crate::metrics::invariants::Invariant;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueFormat {
    /// Render as a double-precision float with exactly 4 decimal places (e.g. 0.2543).
    Float,
    /// Render as an integer (e.g. 4328).
    Integer,
    /// Render as a raw string (e.g. runid "my_runtag").
    Str,
    /// Render as a string wrapped in single quotes (e.g. relstring "'1-0-1'").
    QuotedStr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricValue {
    Float(f64),
    Integer(i64),
    Str(String),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EvaluationType {
    Standard,
    JudgmentGroups,
    Preferences,
}


#[derive(Debug, Clone)]
pub struct EvalConfig {
    pub query_flag: bool,
    /// With `query_flag`, also give the per-topic values of the measures that `trec_eval -q`
    /// leaves out (`gm_map`, `gm_bpref`, `num_q` and `runid`), as pytrec_eval does.
    pub all_query_values: bool,
    pub summary_flag: bool,
    pub relevance_level: i64,
    pub average_complete_flag: bool,
    pub judged_docs_only_flag: bool,
    pub max_num_docs_per_topic: usize,
    /// Number of documents in the collection (`-N`).
    pub num_docs_in_coll: usize,
    pub global_gains: Option<crate::metrics::common::GainsConfig>,
}

impl EvalConfig {
    /// Refuses a negative relevance level.
    pub fn validate(&self) -> Result<(), EvaluationError> {
        if self.relevance_level < 0 {
            return Err(EvaluationError::NegativeRelevanceLevel {
                value: self.relevance_level,
            });
        }
        Ok(())
    }
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            query_flag: false,
            all_query_values: false,
            summary_flag: true,
            relevance_level: 1,
            average_complete_flag: false,
            judged_docs_only_flag: false,
            max_num_docs_per_topic: usize::MAX,
            num_docs_in_coll: 0,
            global_gains: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvalState {
    Standard(QueryEvalState),
    JudgmentGroups(Vec<QueryEvalState>),
}

impl EvalState {
    /// Return the standard QueryEvalState, or the first JG state if in JudgmentGroups mode.
    pub fn get_standard(&self) -> Option<&QueryEvalState> {
        match self {
            EvalState::Standard(q) => Some(q),
            EvalState::JudgmentGroups(jgs) => jgs.first(),
        }
    }
}



pub trait Measure: Send + Sync {
    /// Unique root name of the measure (e.g. "map", "P").
    fn name(&self) -> &'static str;

    /// Short single-sentence summary of the measure (e.g. "Mean Average Precision").
    fn short_description(&self) -> &'static str;

    /// Detailed description/explanation of the measure.
    fn explanation(&self) -> &'static str;

    /// Printing format for the metric's values.
    fn format(&self) -> ValueFormat;

    /// The category of relevance judgments required by this measure.
    fn eval_type(&self) -> EvaluationType;

    /// Returns the exact sub-metric names to be calculated (e.g., `["P_5", "P_10", "P_15"]`).
    fn sub_metrics(&self) -> Vec<String>;

    /// Whether this measure should be reported in the summary averages at the end.
    fn is_summary_enabled(&self) -> bool {
        true
    }

    /// Whether this measure should be reported for individual queries.
    fn is_query_enabled(&self) -> bool {
        true
    }

    /// Whether `calc` reads the topic's ranking and judgments. `trec_eval` checks a topic for
    /// repeated documents only when a measure forms them, so a request for measures that
    /// return false here, such as `num_q`, is never refused for a repeat.
    fn reads_ranking(&self) -> bool {
        true
    }

    /// The invariants this measure claims to uphold (self-documentation,
    /// verified by the uniform invariant test). Defaults to the standard
    /// unit-interval bundle for normalized ranking measures; measures whose
    /// behavior differs (counts, unnormalized scores, signed, string) override.
    fn invariants(&self) -> &'static [Invariant] {
        invariants::STANDARD_UNIT_INTERVAL
    }

    /// Returns the initial values for the running totals of this measure.
    fn initial_values(&self) -> Vec<MetricValue>;

    /// Calculate the score(s) for a single query.
    fn calc(&self, config: &EvalConfig, state: &EvalState) -> Vec<MetricValue>;

    /// Accumulate a single query's scores into a running total.
    fn accumulate(&self, q_scores: &[MetricValue], running_totals: &mut [MetricValue]) {
        for (i, score) in q_scores.iter().enumerate() {
            match (score, &mut running_totals[i]) {
                (MetricValue::Float(s), MetricValue::Float(t)) => *t += s,
                (MetricValue::Integer(s), MetricValue::Integer(t)) => *t += s,
                _ => {} // Strings and non-matching types do not accumulate
            }
        }
    }

    /// Calculate the final summary score from the accumulated totals.
    fn average(&self, config: &EvalConfig, running_totals: &mut [MetricValue], num_queries_evaluated: usize, total_qrels_queries: usize) {
        let denominator = if config.average_complete_flag {
            total_qrels_queries
        } else {
            num_queries_evaluated
        };
        if denominator > 0 {
            for total in running_totals.iter_mut() {
                if let MetricValue::Float(t) = total {
                    *t /= denominator as f64;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::avg_11pt::Avg11PtMeasure;
    use super::common::MeasureParseError;
    use super::iprec_at_recall::IprecAtRecallMeasure;
    use super::map_cut::MapCutMeasure;
    use super::ndcg_cut::NdcgCutMeasure;
    use super::precision::PrecisionCutMeasure;
    use super::precision_avgjg::PrecisionAvgjgMeasure;
    use super::rbp::RbpMeasure;
    use super::rbp_resid::RbpResidMeasure;
    use super::recall::RecallCutMeasure;
    use super::relative_p::RelativePMeasure;
    use super::rprec_mult::RprecMultMeasure;
    use super::rprec_mult_avgjg::RprecMultAvgjgMeasure;
    use super::set_f::SetFMeasure;
    use super::success::SuccessCutMeasure;
    use super::unj::UnjMeasure;
    use super::utility::UtilityMeasure;
    use super::*;

    /// The measures that `new` builds from `sorted` reversed and from `sorted`.
    fn reversed_and_sorted<T: Clone, M: Measure + 'static>(
        new: fn(Vec<T>) -> Result<M, MeasureParseError>,
        sorted: Vec<T>,
    ) -> [Box<dyn Measure>; 2] {
        let reversed = sorted.iter().rev().cloned().collect();
        [
            Box::new(new(reversed).unwrap()),
            Box::new(new(sorted).unwrap()),
        ]
    }

    #[test]
    fn cutoff_constructors_sort_their_cutoffs() {
        for [given, sorted] in [
            reversed_and_sorted(PrecisionCutMeasure::new, vec![2, 5]),
            reversed_and_sorted(RecallCutMeasure::new, vec![2, 5]),
            reversed_and_sorted(SuccessCutMeasure::new, vec![2, 5]),
            reversed_and_sorted(MapCutMeasure::new, vec![2, 5]),
            reversed_and_sorted(NdcgCutMeasure::new, vec![2, 5]),
            reversed_and_sorted(RelativePMeasure::new, vec![2, 5]),
            reversed_and_sorted(UnjMeasure::new, vec![2, 5]),
            reversed_and_sorted(PrecisionAvgjgMeasure::new, vec![2, 5]),
            reversed_and_sorted(IprecAtRecallMeasure::new, vec![0.2, 0.8]),
            reversed_and_sorted(RprecMultMeasure::new, vec![0.5, 2.0]),
            reversed_and_sorted(RprecMultAvgjgMeasure::new, vec![0.5, 2.0]),
        ] {
            assert_eq!(
                given.sub_metrics(),
                sorted.sub_metrics(),
                "{}",
                given.name()
            );
        }
    }

    #[test]
    fn cutoff_constructors_refuse_lists_they_cannot_use() {
        assert_eq!(
            MapCutMeasure::new(vec![0]).err().unwrap().message,
            "rank cutoff must be at least 1"
        );
        assert_eq!(
            PrecisionCutMeasure::new(vec![5, 5]).err().unwrap().message,
            "cutoff 5 is given more than once"
        );
        assert_eq!(
            RprecMultMeasure::new(vec![1.0, 1.0]).err().unwrap().message,
            "cutoff 1 is given more than once"
        );
        // -0 and 0 are equal, though they print differently.
        assert!(IprecAtRecallMeasure::new(vec![-0.0, 0.0]).is_err());
        assert!(PrecisionCutMeasure::new(vec![]).is_err());
        assert!(Avg11PtMeasure::new(vec![], "").is_err());
        assert!(RprecMultMeasure::new(vec![f64::NAN]).is_err());
        // Distinct recall endpoints and repeated averaging levels are legal.
        IprecAtRecallMeasure::new(vec![0.0, 1.0]).unwrap();
        Avg11PtMeasure::new(vec![0.0, 0.0, 1.0], "").unwrap();
        RprecMultMeasure::new(vec![f64::MIN_POSITIVE, 1e300]).unwrap();
    }

    #[test]
    fn parameter_constructors_refuse_values_outside_their_domain() {
        assert!(SetFMeasure::new(f64::NAN, "").is_err());
        assert!(UtilityMeasure::new(vec![f64::INFINITY, 0.0, 0.0, 0.0], "").is_err());
        assert!(UtilityMeasure::new(vec![], "").is_err());
        for p in [0.0, 1.0 - f64::EPSILON] {
            RbpMeasure::new(&format!("p={p}")).unwrap();
            RbpResidMeasure::new(&format!("p={p}")).unwrap();
        }
        SetFMeasure::new(0.0, "").unwrap();
        UtilityMeasure::new(vec![1e300, -1e300, 0.0, 1.0], "").unwrap();
    }
}
