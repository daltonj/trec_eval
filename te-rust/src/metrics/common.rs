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
/// ascending order, which the single-pass measures rely on. A cutoff must fit
/// in an `i64`, as trec_eval's `long` cutoffs must.
/// An empty string yields the provided defaults.
pub fn parse_int_cutoffs(params: &str, default: &[usize]) -> Result<Vec<usize>, MeasureParseError> {
    if params.is_empty() {
        return Ok(default.to_vec());
    }
    let mut list = Vec::new();
    for s in params.split(',') {
        let tok = s.trim();
        // As trec_eval reads a cutoff: the leading integer, so `5x` and `1.5` read as 5 and 1,
        // with a warning for take_parse_warnings.
        let invalid = || MeasureParseError::new(format!("invalid integer cutoff '{}'", tok));
        let (v, partial) = match crate::io::parsers::read_leading_integer(tok) {
            Ok(read) => read,
            Err(std::num::IntErrorKind::PosOverflow | std::num::IntErrorKind::NegOverflow) => {
                let message = format!("integer cutoff '{}' is out of range", tok);
                return Err(MeasureParseError::new(message));
            }
            Err(_) => return Err(invalid()),
        };
        if partial {
            parse_warning(format!("value '{}' partially read as '{}'", tok, v));
        }
        list.push(usize::try_from(v).map_err(|_| invalid())?);
    }
    checked_rank_cutoffs(list)
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
/// carries meaning (`utility`) or that may repeat a value (`11pt_avg`) use [`parse_float_cutoffs`].
pub fn parse_sorted_float_cutoffs(params: &str, default: &[f64]) -> Result<Vec<f64>, MeasureParseError> {
    checked_float_cutoffs(parse_float_cutoffs(params, default)?)
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

thread_local! {
    static PARSE_WARNINGS: std::cell::RefCell<Vec<String>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

fn parse_warning(message: String) {
    PARSE_WARNINGS.with(|warnings| warnings.borrow_mut().push(message));
}

/// Takes the warnings that measure parameters raised on this thread since the last call, such as
/// "value '5x' partially read as '5'" for `P.5x`. [`resolve_measures`] clears them before it
/// starts, so after it returns they belong to that call.
///
/// [`resolve_measures`]: crate::metrics::registry::resolve_measures
pub fn take_parse_warnings() -> Vec<String> {
    PARSE_WARNINGS.with(|warnings| std::mem::take(&mut *warnings.borrow_mut()))
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

fn require_cutoffs<T>(cutoffs: &[T]) -> Result<(), MeasureParseError> {
    if cutoffs.is_empty() {
        return Err(MeasureParseError::new("at least one cutoff is required"));
    }
    Ok(())
}

fn checked_rank_cutoffs(mut cutoffs: Vec<usize>) -> Result<Vec<usize>, MeasureParseError> {
    require_cutoffs(&cutoffs)?;
    cutoffs.sort_unstable();
    if cutoffs.first() == Some(&0) {
        return Err(MeasureParseError::new("rank cutoff must be at least 1"));
    }
    if cutoffs.iter().any(|&cutoff| i64::try_from(cutoff).is_err()) {
        return Err(MeasureParseError::new("integer cutoff is out of range"));
    }
    if cutoffs.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(MeasureParseError::new("duplicate cutoffs"));
    }
    Ok(cutoffs)
}

fn checked_float_cutoffs(mut list: Vec<f64>) -> Result<Vec<f64>, MeasureParseError> {
    require_cutoffs(&list)?;
    for &value in &list {
        finite_float(value)?;
    }
    list.sort_by(f64::total_cmp);
    if list.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(MeasureParseError::new("duplicate cutoffs"));
    }
    // Each row is named with the cutoff to two decimals (`Rprec_mult_1.00`). Rounding is
    // monotone, so in the sorted list two cutoffs that print the same name are neighbours.
    let label = |cutoff: f64| format!("{:.2}", cutoff);
    if let Some(pair) = list
        .windows(2)
        .find(|pair| label(pair[0]) == label(pair[1]))
    {
        return Err(MeasureParseError::new(format!(
            "cutoffs {} and {} both print as {}",
            pair[0],
            pair[1],
            label(pair[0])
        )));
    }
    Ok(list)
}

/// Sorts rank cutoffs and applies the same checks as the registry parser.
///
/// # Panics
///
/// Panics on an empty list, zero, repeated cutoffs or a cutoff larger than i64::MAX.
pub(crate) fn sorted_rank_cutoffs(cutoffs: Vec<usize>) -> Vec<usize> {
    checked_rank_cutoffs(cutoffs).expect("invalid rank cutoffs")
}

/// Sorts float cutoffs and applies the same checks as the registry parser.
///
/// # Panics
///
/// Panics on an empty list, non-finite cutoffs or cutoffs that print the same row name.
pub(crate) fn sorted_float_cutoffs(cutoffs: Vec<f64>) -> Vec<f64> {
    checked_float_cutoffs(cutoffs).expect("invalid float cutoffs")
}

/// Refuse recall levels outside [0, 1], for `iprec_at_recall` and `11pt_avg`.
pub(crate) fn recall_levels(levels: &[f64]) -> Result<(), MeasureParseError> {
    require_cutoffs(levels)?;
    for &level in levels {
        finite_float(level)?;
        if !(0.0..=1.0).contains(&level) {
            return Err(MeasureParseError::new(format!(
                "recall level {} is outside [0, 1]",
                level
            )));
        }
    }
    Ok(())
}

/// Refuse multiples of R that are not positive and finite.
pub(crate) fn r_multiples(multiples: &[f64]) -> Result<(), MeasureParseError> {
    for &multiple in multiples {
        finite_float(multiple)?;
        if multiple <= 0.0 {
            return Err(MeasureParseError::new(format!(
                "multiple of R {} is not a positive number",
                multiple
            )));
        }
    }
    Ok(())
}

pub(crate) fn rbp_persistence(p: f64) -> Result<(), MeasureParseError> {
    finite_float(p)?;
    if !(0.0..1.0).contains(&p) {
        return Err(MeasureParseError::new(format!(
            "persistence p {} is outside [0, 1)",
            p
        )));
    }
    Ok(())
}

pub(crate) fn set_f_weight(weight: f64) -> Result<(), MeasureParseError> {
    finite_float(weight)?;
    if weight < 0.0 {
        return Err(MeasureParseError::new(format!(
            "set_F weight {} is negative",
            weight
        )));
    }
    Ok(())
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
    #[cfg(target_pointer_width = "64")]
    fn int_cutoffs_must_fit_in_i64() {
        let max = parse_int_cutoffs("9223372036854775807", &[]).unwrap();
        assert_eq!(max, vec![i64::MAX as usize]);
        for bad in ["9223372036854775808", "18446744073709551615", "-9223372036854775809"] {
            let err = parse_int_cutoffs(bad, &[]).unwrap_err();
            assert!(err.message.contains("out of range"), "{bad}: {}", err.message);
        }
    }

    #[test]
    fn int_cutoffs_read_the_leading_integer_with_a_warning() {
        take_parse_warnings();
        assert_eq!(parse_int_cutoffs("1.5,5x,20", &[]).unwrap(), vec![1, 5, 20]);
        assert_eq!(
            take_parse_warnings(),
            ["value '1.5' partially read as '1'", "value '5x' partially read as '5'"]
        );
        assert_eq!(parse_int_cutoffs("5,20", &[]).unwrap(), vec![5, 20]);
        assert!(take_parse_warnings().is_empty());
        for bad in [".5", "x5", "+", "-1", "9223372036854775808"] {
            assert!(parse_int_cutoffs(bad, &[]).is_err(), "{bad}");
        }
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
    fn sorted_float_cutoffs_refuse_names_that_collide() {
        for bad in ["0.999,1.001", "0.101,0.104", "0.005,0.01", "1.0,0.996"] {
            let err = parse_sorted_float_cutoffs(bad, &[]).unwrap_err();
            assert!(err.message.contains("both print as"), "{bad}: {}", err.message);
        }
        // 0.105 is stored just below 0.105 and prints as 0.10, 0.106 as 0.11.
        let distinct = parse_sorted_float_cutoffs("0.106,0.105", &[]).unwrap();
        assert_eq!(distinct, vec![0.105, 0.106]);
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
