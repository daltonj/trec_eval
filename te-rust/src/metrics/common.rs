use crate::metrics::EvalConfig;
use std::collections::HashMap;
use std::fmt;

/// Error returned when a measure's parameter string cannot be parsed.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasureParseError {
    pub message: String,
    /// The measure name itself is illegal, as opposed to its parameters.
    pub illegal_name: bool,
}

impl MeasureParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), illegal_name: false }
    }

    pub fn illegal_name(message: impl Into<String>) -> Self {
        Self { message: message.into(), illegal_name: true }
    }
}

impl fmt::Display for MeasureParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MeasureParseError {}

/// Parse a comma-separated list of integer cutoffs (e.g. "5,10,20") into
/// ascending order, which the single-pass measures rely on.
/// An empty string yields the provided defaults.
pub fn parse_int_cutoffs(params: &str, default: &[usize]) -> Result<Vec<usize>, MeasureParseError> {
    if params.is_empty() {
        return Ok(default.to_vec());
    }
    let mut list = Vec::new();
    for s in params.split(',') {
        let tok = s.trim();
        match tok.parse::<usize>() {
            Ok(v) => list.push(v),
            Err(_) => return Err(MeasureParseError::new(format!("invalid integer cutoff '{}'", tok))),
        }
    }
    list.sort_unstable();
    if list.first() == Some(&0) {
        return Err(MeasureParseError::new("rank cutoff must be at least 1"));
    }
    if list.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(MeasureParseError::new("duplicate cutoffs"));
    }
    Ok(list)
}

/// Parse a comma-separated list of float cutoffs/coefficients (e.g. "0.0,0.5,1.0").
/// Each value must be a finite number. An empty string yields the provided defaults.
pub fn parse_float_cutoffs(params: &str, default: &[f64]) -> Result<Vec<f64>, MeasureParseError> {
    if params.is_empty() {
        return Ok(default.to_vec());
    }
    let mut list = Vec::new();
    for s in params.split(',') {
        list.push(parse_finite_float(s.trim())?);
    }
    Ok(list)
}

/// Parse a float parameter that must be a finite number. NaN, infinity, and values that
/// overflow to infinity (`1e400`) are refused.
pub(crate) fn parse_finite_float(tok: &str) -> Result<f64, MeasureParseError> {
    let value = tok
        .parse::<f64>()
        .map_err(|_| MeasureParseError::new(format!("invalid float value '{}'", tok)))?;
    finite_float(value)
}

pub(crate) fn finite_float(value: f64) -> Result<f64, MeasureParseError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(MeasureParseError::new(format!(
            "float value '{}' is not a finite number",
            value
        )))
    }
}

pub(crate) fn utility_coefficients(coeffs: &[f64]) -> Result<(), MeasureParseError> {
    if coeffs.len() != 4 {
        return Err(MeasureParseError::new(
            "utility requires exactly 4 coefficients",
        ));
    }
    for &value in coeffs {
        finite_float(value)?;
    }
    Ok(())
}

/// Parse a comma-separated list of float cutoffs into ascending order, for
/// measures that walk the list in step with the ranking. Lists whose order
/// carries meaning (`11pt_avg`, `utility`) use [`parse_float_cutoffs`].
pub fn parse_sorted_float_cutoffs(params: &str, default: &[f64]) -> Result<Vec<f64>, MeasureParseError> {
    let mut list = parse_float_cutoffs(params, default)?;
    list.sort_by(f64::total_cmp);
    if list.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(MeasureParseError::new("duplicate cutoffs"));
    }
    Ok(list)
}

/// Parse a comma-separated list of `key=value` pairs (e.g. "p=0.9").
/// Values are returned as raw strings for the caller to interpret. An empty
/// string yields an empty list.
pub fn parse_key_values(params: &str) -> Result<Vec<(String, String)>, MeasureParseError> {
    if params.is_empty() {
        return Ok(Vec::new());
    }
    let mut pairs = Vec::new();
    for part in params.split(',') {
        let (k, v) = part
            .split_once('=')
            .ok_or_else(|| MeasureParseError::new(format!("expected key=value, got '{}'", part.trim())))?;
        pairs.push((k.trim().to_string(), v.trim().to_string()));
    }
    Ok(pairs)
}

/// Helper function to count the number of relevant documents retrieved up to a certain rank limit.
pub(crate) fn count_relevant_retrieved_up_to(results_rel_list: &[i64], limit: usize, config: &EvalConfig) -> usize {
    let actual_limit = std::cmp::min(limit, results_rel_list.len());
    results_rel_list[..actual_limit]
        .iter()
        .filter(|&&rel| rel >= config.relevance_level)
        .count()
}

#[derive(Debug, Clone, Default)]
pub struct GainsConfig {
    pub custom_gains: HashMap<i64, f64>,
}

impl GainsConfig {
    /// Parse a gain map such as `1=3,2=9`. Every name must be an integer relevance level,
    /// given once, and every gain a finite number. An empty string yields an empty map.
    pub fn parse(params_str: &str) -> Result<Self, MeasureParseError> {
        let mut custom_gains = HashMap::new();
        for (name, value) in parse_key_values(params_str)? {
            let rel_level = name.parse::<i64>().map_err(|_| {
                MeasureParseError::new(format!("invalid relevance level '{}'", name))
            })?;
            let gain = value
                .parse::<f64>()
                .ok()
                .filter(|gain| gain.is_finite())
                .ok_or_else(|| {
                    MeasureParseError::new(format!(
                        "invalid gain '{}' for relevance level {}",
                        value, rel_level
                    ))
                })?;
            if custom_gains.insert(rel_level, gain).is_some() {
                return Err(MeasureParseError::new(format!(
                    "relevance level {} is given more than once",
                    rel_level
                )));
            }
        }
        Ok(Self { custom_gains })
    }
}

#[derive(Debug, Clone)]
pub struct RelGain {
    pub rel_level: i64,
    pub gain: f64,
    pub num_at_level: usize,
}

#[derive(Debug, Clone)]
pub struct Gains {
    pub rel_gains: Vec<RelGain>,
    pub _total_num_at_levels: usize,
}

impl Gains {
    pub fn setup(config: &GainsConfig, rel_levels: &[usize]) -> Self {
        let mut rel_gains = Vec::new();

        // 1. First add the custom gains specified in parameters
        for (&rel_level, &gain) in &config.custom_gains {
            let num_at_level = if rel_level >= 0 && (rel_level as usize) < rel_levels.len() {
                rel_levels[rel_level as usize]
            } else {
                0
            };
            rel_gains.push(RelGain {
                rel_level,
                gain,
                num_at_level,
            });
        }

        // 2. Then add all other relevance levels that were NOT specified in custom_gains,
        // up to the highest level with a judged document. rel_levels can be padded past it.
        let num_rel_levels = rel_levels.iter().rposition(|&n| n > 0).map_or(0, |top| top + 1);
        for i in 0..num_rel_levels {
            let rel_level = i as i64;
            if !config.custom_gains.contains_key(&rel_level) {
                rel_gains.push(RelGain {
                    rel_level,
                    gain: rel_level as f64,
                    num_at_level: rel_levels[i],
                });
            }
        }

        // 3. Sort by increasing gain value using total_cmp for determinism
        rel_gains.sort_by(|a, b| a.gain.total_cmp(&b.gain));

        let _total_num_at_levels = rel_gains.iter().map(|g| g.num_at_level).sum();

        Gains {
            rel_gains,
            _total_num_at_levels,
        }
    }

    pub fn get_gain(&self, rel_level: i64) -> f64 {
        for g in &self.rel_gains {
            if g.rel_level == rel_level {
                return g.gain;
            }
        }
        0.0
    }
}

#[cfg(test)]
pub(crate) fn make_mock_state(results_rel_list: Vec<i64>, num_rel: i64) -> crate::eval::alignment::QueryEvalState {
    let num_ret = results_rel_list.len();
    let num_rel_ret = results_rel_list.iter().filter(|&&r| r >= 1).count(); // assume relevance level cutoff >= 1
    
    let mut rel_levels = vec![0; 2];
    rel_levels[1] = num_rel as usize;
    rel_levels[0] = 100; // default count of non-relevant docs
    
    crate::eval::alignment::QueryEvalState {
        qid: "mock_topic".to_string(),
        run_id: "mock_run".to_string(),
        results_rel_list,
        num_ret,
        num_rel: num_rel as usize,
        num_rel_ret,
        num_nonpool: 0,
        num_unjudged_in_pool: 0,
        rel_levels,
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn int_cutoffs_empty_uses_default() {
        assert_eq!(parse_int_cutoffs("", &[5, 10, 20]).unwrap(), vec![5, 10, 20]);
    }

    #[test]
    fn int_cutoffs_parses_and_trims() {
        assert_eq!(parse_int_cutoffs("5, 10 ,20", &[]).unwrap(), vec![5, 10, 20]);
    }

    #[test]
    fn int_cutoffs_rejects_bad_token() {
        assert!(parse_int_cutoffs("5,x,20", &[]).is_err());
        assert!(parse_int_cutoffs("5,-1", &[]).is_err());
    }

    #[test]
    fn float_cutoffs_empty_uses_default() {
        assert_eq!(parse_float_cutoffs("", &[0.0, 1.0]).unwrap(), vec![0.0, 1.0]);
    }

    #[test]
    fn float_cutoffs_parses_and_trims() {
        assert_eq!(parse_float_cutoffs("0.0, 0.5 ,1.0", &[]).unwrap(), vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn float_cutoffs_rejects_bad_token() {
        assert!(parse_float_cutoffs("0.0,nope", &[]).is_err());
    }

    #[test]
    fn float_cutoffs_rejects_non_finite_values() {
        for bad in ["nan", "NaN", "inf", "-inf", "infinity", "1e400", "0.5,nan"] {
            assert!(parse_float_cutoffs(bad, &[]).is_err(), "{bad}");
        }
        assert_eq!(parse_float_cutoffs("1e300,-0.5", &[]).unwrap(), vec![1e300, -0.5]);
    }

    #[test]
    fn key_values_empty_is_empty() {
        assert_eq!(parse_key_values("").unwrap(), Vec::<(String, String)>::new());
    }

    #[test]
    fn key_values_parses_and_trims() {
        assert_eq!(
            parse_key_values("p=0.9, q = 2").unwrap(),
            vec![("p".to_string(), "0.9".to_string()), ("q".to_string(), "2".to_string())]
        );
    }

    #[test]
    fn key_values_rejects_missing_equals() {
        assert!(parse_key_values("p0.9").is_err());
    }

    #[test]
    fn gains_parse_levels_and_gains() {
        let gains = GainsConfig::parse("1=3, 2 = 9.5,-1=0").unwrap().custom_gains;
        assert_eq!(gains, HashMap::from([(1, 3.0), (2, 9.5), (-1, 0.0)]));
        assert!(GainsConfig::parse("").unwrap().custom_gains.is_empty());
    }

    #[test]
    fn gains_refuse_what_they_cannot_use() {
        for bad in ["1=x", "x=1", "1=nan", "1=-inf", "5,5", "1=2,1=5"] {
            assert!(GainsConfig::parse(bad).is_err(), "{bad}");
        }
    }
}
