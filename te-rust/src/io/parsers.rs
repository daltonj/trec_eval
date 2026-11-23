#![allow(dead_code)]
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The file has no lines at all.
    EmptyFile,
    /// The file has lines, but each is blank or a comment.
    NoDataLines,
    /// A line holds a NUL byte.
    NulCharacter {
        line_num: usize,
    },
    /// A line holds a CR that is not followed by LF, as in a file with CR line endings.
    CarriageReturn {
        line_num: usize,
    },
    /// A data line is not UTF-8; `column` is the 1-based offset of the first bad byte.
    InvalidUtf8 { line_num: usize, column: usize },
    InvalidFieldCount { line_num: usize, expected: FieldCount, found: usize },
    InvalidInteger { line_num: usize, field: String, value: String },
    InvalidFloat { line_num: usize, field: String, value: String },
    DuplicateDocument { line_num: usize, qid: String, docno: String },
    RelevanceOutOfBounds { line_num: usize, value: i64 },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::EmptyFile => write!(f, "File is empty"),
            ParseError::NoDataLines => write!(f, "No data lines"),
            ParseError::NulCharacter { line_num } => write!(f, "Line {}: NUL character", line_num),
            ParseError::CarriageReturn { line_num } => {
                write!(f, "Line {}: CR character without LF", line_num)
            }
            ParseError::InvalidUtf8 { line_num, column } => {
                write!(f, "Line {}: invalid UTF-8 at byte {}", line_num, column)
            }
            ParseError::InvalidFieldCount { line_num, expected, found } => {
                write!(f, "Line {}: expected {} fields, found {}", line_num, expected, found)
            }
            ParseError::InvalidInteger { line_num, field, value } => {
                write!(f, "Line {}: invalid integer for field '{}': '{}'", line_num, field, value)
            }
            ParseError::InvalidFloat { line_num, field, value } => {
                write!(f, "Line {}: invalid float for field '{}': '{}'", line_num, field, value)
            }
            ParseError::DuplicateDocument { line_num, qid, docno } => {
                write!(f, "Line {}: duplicate document '{}' for query '{}'", line_num, docno, qid)
            }
            ParseError::RelevanceOutOfBounds { line_num, value } => write!(
                f,
                "Line {}: relevance level {} is outside {} to {}",
                line_num,
                value,
                REL_LEVELS.start(),
                REL_LEVELS.end()
            ),
        }
    }
}

impl std::error::Error for ParseError {}

/// The number of fields a line must have: exactly 4 in a qrels file, at least 6 in a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldCount {
    Exactly(usize),
    AtLeast(usize),
}

impl std::fmt::Display for FieldCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldCount::Exactly(n) => write!(f, "{}", n),
            FieldCount::AtLeast(n) => write!(f, "at least {}", n),
        }
    }
}

#[derive(Debug)]
pub enum IOError {
    FileReadError(io::Error),
    FormatError(ParseError),
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IOError::FileReadError(e) => write!(f, "I/O Error: {}", e),
            IOError::FormatError(e) => write!(f, "Format Error: {}", e),
        }
    }
}

impl std::error::Error for IOError {}

/// Reads the leading integer of `value` as `trec_eval`'s `str_to_long` does: C-locale whitespace,
/// an optional sign, then decimal digits. Returns the integer and whether characters follow it, so
/// `1.5` reads as `(1, true)`. A value with no leading digit is `InvalidDigit`, and an integer that
/// does not fit in an `i64` is `PosOverflow` or `NegOverflow`.
pub fn read_leading_integer(value: &str) -> Result<(i64, bool), std::num::IntErrorKind> {
    let number = value.trim_start_matches(is_field_sep);
    let start = usize::from(number.starts_with(['+', '-']));
    let digits = number[start..]
        .bytes()
        .take_while(u8::is_ascii_digit)
        .count();
    if digits == 0 {
        return Err(std::num::IntErrorKind::InvalidDigit);
    }
    let end = start + digits;
    // A sign and digits can fail to parse only by overflowing.
    let integer = number[..end].parse::<i64>().map_err(|_| {
        if number.starts_with('-') {
            std::num::IntErrorKind::NegOverflow
        } else {
            std::num::IntErrorKind::PosOverflow
        }
    })?;
    Ok((integer, end < number.len()))
}

/// The relevance levels a qrels or qrels_jg file may use: -127 to 127, as `trec_eval`'s `get_qrels`
/// and `get_qrels_jg` document. The highest level of a topic sizes its table of grade counts, so the
/// upper bound also limits that allocation.
pub const REL_LEVELS: std::ops::RangeInclusive<i64> = -127..=127;

/// Reads a grade as `trec_eval`'s `str_to_long` does: the leading integer, with a warning when
/// characters follow it, so `1.5` reads as 1 and `0.9` as 0. A value with no leading integer, or
/// one outside [`REL_LEVELS`], is an error.
fn parse_grade(value: &str, line_num: usize, warnings: &mut Vec<String>) -> Result<i64, IOError> {
    let (rel, partial) = read_leading_integer(value).map_err(|_| {
        IOError::FormatError(ParseError::InvalidInteger {
            line_num,
            field: "rel".to_string(),
            value: value.to_string(),
        })
    })?;
    if partial {
        warnings.push(format!(
            "Line {}: value '{}' partially read as '{}'",
            line_num, value, rel
        ));
    }
    if !REL_LEVELS.contains(&rel) {
        return Err(IOError::FormatError(ParseError::RelevanceOutOfBounds {
            line_num,
            value: rel,
        }));
    }
    Ok(rel)
}

/// Reads a score with `strtod`, as `trec_eval` does: the longest leading number, with a warning
/// when characters follow it, so `1.5x` reads as 1.5. Infinities and hexadecimal values such as
/// `0x1p3` are read as `strtod` reads them. A value outside the range of an `f64`, such as `1e400`
/// or `1e-400`, is kept as ±inf or ±0 with a warning. NaN, and a value with no leading number, are
/// errors.
fn parse_score(value: &str, line_num: usize, warnings: &mut Vec<String>) -> Result<f64, IOError> {
    let (sim, len) = strtod_prefix(value).ok_or_else(|| {
        IOError::FormatError(ParseError::InvalidFloat {
            line_num,
            field: "sim".to_string(),
            value: value.to_string(),
        })
    })?;
    if out_of_range(&value[..len], sim) {
        warnings.push(format!(
            "Line {}: score '{}' out of range, read as {}",
            line_num, value, sim
        ));
    }
    if len < value.len() {
        warnings.push(format!(
            "Line {}: score '{}' partially read as '{}'",
            line_num,
            value,
            &value[..len]
        ));
    }
    Ok(sim)
}

/// Whether `strtod` sets `ERANGE` for the number `prefix` and returns its `value` of ±inf or ±0:
/// a finite number too large or too small for an `f64`. Some C libraries also set `ERANGE` for a
/// subnormal result, which is close to the number given; neither trec_eval nor te-rust warns then.
fn out_of_range(prefix: &str, value: f64) -> bool {
    let unsigned = prefix.trim_start_matches(['+', '-']);
    if value.is_infinite() {
        return !unsigned.starts_with(['i', 'I']);
    }
    if value != 0.0 {
        return false;
    }
    let (significand, exponent_marks) = match unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        Some(hex) => (hex, ['p', 'P']),
        None => (unsigned, ['e', 'E']),
    };
    let significand = significand.split(exponent_marks).next().unwrap_or_default();
    significand
        .bytes()
        .any(|b| b.is_ascii_hexdigit() && b != b'0')
}

/// The value and length of the longest prefix of `s` that C's `strtod` reads in the C locale as a
/// decimal, hexadecimal or infinite number, or `None` when there is none, as for a NaN.
fn strtod_prefix(s: &str) -> Option<(f64, usize)> {
    let bytes = s.as_bytes();
    let sign = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let negative = bytes.first() == Some(&b'-');
    let rest = &bytes[sign..];
    let has_word =
        |word: &[u8]| rest.len() >= word.len() && rest[..word.len()].eq_ignore_ascii_case(word);
    let signed = |x: f64| if negative { -x } else { x };
    if has_word(b"infinity") {
        return Some((signed(f64::INFINITY), sign + 8));
    }
    if has_word(b"inf") {
        return Some((signed(f64::INFINITY), sign + 3));
    }
    if let Some(hex) = rest
        .strip_prefix(b"0x")
        .or_else(|| rest.strip_prefix(b"0X"))
    {
        if let Some((value, len)) = hex_prefix(hex) {
            return Some((signed(value), sign + 2 + len));
        }
    }
    let int_digits = count_digits(rest);
    let mut len = int_digits;
    if rest.get(len) == Some(&b'.') {
        let frac_digits = count_digits(&rest[len + 1..]);
        if int_digits + frac_digits > 0 {
            len += 1 + frac_digits;
        }
    }
    if len == 0 {
        return None;
    }
    len += exponent_len(&rest[len..], b"eE");
    // Rust's parser rounds as strtod does, and gives an infinity or zero out of range.
    let value = s[..sign + len].parse::<f64>().ok()?;
    Some((value, sign + len))
}

/// The value and length of a hexadecimal significand and optional binary exponent, as `strtod`
/// reads them after `0x`, or `None` when no hexadecimal digit follows.
fn hex_prefix(rest: &[u8]) -> Option<(f64, usize)> {
    let mut mantissa: u64 = 0;
    let mut exponent: i64 = 0; // the value is mantissa * 2^exponent
    let mut sticky = false; // a nonzero digit did not fit in the mantissa
    let mut digits = 0;
    let mut point = false;
    let mut len = 0;
    while let Some(&c) = rest.get(len) {
        if c == b'.' && !point {
            point = true;
        } else if let Some(d) = char::from(c).to_digit(16) {
            digits += 1;
            if mantissa >> 60 == 0 {
                mantissa = mantissa << 4 | u64::from(d);
                if point {
                    exponent -= 4;
                }
            } else {
                sticky |= d != 0;
                if !point {
                    exponent += 4;
                }
            }
        } else {
            break;
        }
        len += 1;
    }
    if digits == 0 {
        return None;
    }
    let exp_len = exponent_len(&rest[len..], b"pP");
    if exp_len > 0 {
        let (negative, digits) = match &rest[len + 1..len + exp_len] {
            [b'-', digits @ ..] => (true, digits),
            [b'+', digits @ ..] => (false, digits),
            digits => (false, digits),
        };
        let value = digits.iter().fold(0i64, |e, &c| {
            e.saturating_mul(10).saturating_add(i64::from(c - b'0'))
        });
        exponent = exponent.saturating_add(if negative { -value } else { value });
        len += exp_len;
    }
    Some((round_binary(mantissa, sticky, exponent), len))
}

/// The number of ASCII digits at the start of `bytes`.
fn count_digits(bytes: &[u8]) -> usize {
    bytes.iter().take_while(|c| c.is_ascii_digit()).count()
}

/// The length of the exponent at the start of `bytes`, as `strtod` reads it: a letter in `marks`,
/// an optional sign and at least one digit. 0 when there is none.
fn exponent_len(bytes: &[u8], marks: &[u8; 2]) -> usize {
    if !bytes.first().is_some_and(|b| marks.contains(b)) {
        return 0;
    }
    let sign = usize::from(matches!(bytes.get(1), Some(b'+' | b'-')));
    match count_digits(&bytes[1 + sign..]) {
        0 => 0,
        digits => 1 + sign + digits,
    }
}

/// `mantissa * 2^exponent` rounded to the nearest `f64`, ties to even, where `sticky` means that
/// nonzero bits below the mantissa were dropped.
fn round_binary(mantissa: u64, sticky: bool, exponent: i64) -> f64 {
    if mantissa == 0 {
        return 0.0;
    }
    // m holds the significand with its leading 1 at bit 63.
    let zeros = mantissa.leading_zeros();
    let m = u128::from(mantissa << zeros);
    // The value lies in [2^top, 2^(top + 1)).
    let top = exponent.saturating_add(63 - i64::from(zeros));
    if top > 1023 {
        return f64::INFINITY;
    }
    if top < -1075 {
        return 0.0;
    }
    // Keep the 53 bits of a normal f64, or one bit fewer for each binary place below 2^-1022.
    let shift = (11 + (-1022 - top).max(0)) as u32;
    // Round the dropped bits r to nearest, ties to even; `sticky` counts as a nonzero bit below r.
    let (q, r, half) = (m >> shift, m & ((1u128 << shift) - 1), 1u128 << (shift - 1));
    let up = r > half || (r == half && (sticky || q & 1 == 1));
    let q = (q + u128::from(up)) as u64;
    if top < -1022 {
        return f64::from_bits(q); // subnormal; a carry into bit 52 gives the smallest normal
    }
    // Rounding up can carry into a 54th bit.
    let (q, top) = if q >> 53 == 1 {
        (q >> 1, top + 1)
    } else {
        (q, top)
    };
    if top > 1023 {
        return f64::INFINITY;
    }
    f64::from_bits(((top + 1023) as u64) << 52 | (q & ((1u64 << 52) - 1)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrelsRecord {
    pub docno: String,
    pub rel: i64,
}

#[derive(Debug, Clone)]
pub struct QrelsQuery {
    pub qid: String,
    pub records: Vec<QrelsRecord>,
}

#[derive(Debug, Clone)]
pub struct QrelsData {
    pub queries: Vec<QrelsQuery>,
    pub comments: Vec<String>,
    /// Problems that did not stop parsing, such as a grade read only in part.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrelsJGGroup {
    pub jg: String,
    pub records: Vec<QrelsRecord>,
}

#[derive(Debug, Clone)]
pub struct QrelsJGQuery {
    pub qid: String,
    pub groups: Vec<QrelsJGGroup>,
}

#[derive(Debug, Clone)]
pub struct QrelsJGData {
    pub queries: Vec<QrelsJGQuery>,
    pub comments: Vec<String>,
    /// Problems that did not stop parsing, such as a grade read only in part.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunRecord {

    pub docno: String,
    pub sim: f64,
}

#[derive(Debug, Clone)]
pub struct RunQuery {
    pub qid: String,
    pub records: Vec<RunRecord>,
}

#[derive(Debug, Clone)]
pub struct RunData {
    pub run_id: String,
    pub queries: Vec<RunQuery>,
    pub comments: Vec<String>,
    /// Problems that did not stop parsing, such as a score read only in part.
    pub warnings: Vec<String>,
}

struct RawQrels {
    qid: String,
    docno: String,
    rel: i64,
}

struct RawRun {
    qid: String,
    docno: String,
    sim: f64,
    run_id: String,
}

/// True for the field separators of trec_eval's parsers: C `isspace` in the C locale.
/// `char::is_whitespace` also matches non-ASCII characters such as U+00A0, which
/// can occur inside a docno.
fn is_field_sep(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\x0B' | '\x0C' | '\r')
}

fn split_fields(line: &str) -> Vec<&str> {
    line.split(is_field_sep)
        .filter(|field| !field.is_empty())
        .collect()
}

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

/// Reads lines as `BufRead::lines` does, numbered from 1, and checks each line's bytes with
/// `decode_line`. A UTF-8 byte-order mark at the start of the file is removed; elsewhere it is part
/// of the line. A file with no lines is an `EmptyFile` error.
fn read_lines(mut reader: impl BufRead) -> impl Iterator<Item = Result<(usize, String), IOError>> {
    let mut line_num = 0;
    let mut lines = std::iter::from_fn(move || {
        let mut bytes = Vec::new();
        match reader.read_until(b'\n', &mut bytes) {
            Ok(0) => None,
            Ok(_) => {
                line_num += 1;
                let byte_offset = if line_num == 1 && bytes.starts_with(UTF8_BOM) {
                    bytes.drain(..UTF8_BOM.len());
                    UTF8_BOM.len()
                } else {
                    0
                };
                Some(decode_line(bytes, line_num, byte_offset).map(|line| (line_num, line)))
            }
            Err(e) => Some(Err(IOError::FileReadError(e))),
        }
    })
    .peekable();
    let empty = lines
        .peek()
        .is_none()
        .then_some(Err(IOError::FormatError(ParseError::EmptyFile)));
    empty.into_iter().chain(lines)
}

/// Removes the line ending (LF, CRLF, or a CR that ends the file) and refuses bytes that would make
/// the line's fields wrong: a NUL, which trec_eval's file formats do not allow, and any other CR,
/// which would join the lines of a file with CR line endings. A data line must be UTF-8. A comment
/// is never compared, so its invalid bytes are replaced rather than refused.
fn decode_line(mut bytes: Vec<u8>, line_num: usize, byte_offset: usize) -> Result<String, IOError> {
    if bytes.ends_with(b"\n") {
        bytes.pop();
    }
    if bytes.ends_with(b"\r") {
        bytes.pop();
    }
    if bytes.contains(&0) {
        return Err(IOError::FormatError(ParseError::NulCharacter { line_num }));
    }
    if bytes.contains(&b'\r') {
        return Err(IOError::FormatError(ParseError::CarriageReturn {
            line_num,
        }));
    }
    let start = bytes
        .iter()
        .position(|&b| !is_field_sep(char::from(b)))
        .unwrap_or(bytes.len());
    if bytes[start..].starts_with(b"#") {
        return Ok(String::from_utf8_lossy(&bytes).into_owned());
    }
    String::from_utf8(bytes).map_err(|e| {
        let column = byte_offset + e.utf8_error().valid_up_to() + 1;
        IOError::FormatError(ParseError::InvalidUtf8 { line_num, column })
    })
}

/// Reads the data lines of a qrels or qrels_jg file, `qid  iter_or_jg  docno  rel`, into records
/// made by `record` from the line number, the first three fields and the grade. Comments are added
/// to `comments`, and each blank line adds a warning. A file without data lines is an error.
fn read_judgments<T>(
    reader: impl BufRead,
    comments: &mut Vec<String>,
    warnings: &mut Vec<String>,
    mut record: impl FnMut(usize, [&str; 3], i64) -> T,
) -> Result<Vec<T>, IOError> {
    let mut records = Vec::new();
    for line in read_lines(reader) {
        let (line_num, line) = line?;
        let trimmed = line.trim_matches(is_field_sep);
        if trimmed.is_empty() {
            warnings.push(format!("Line {}: blank line skipped", line_num));
            continue;
        }
        if let Some(comment) = trimmed.strip_prefix('#') {
            comments.push(comment.to_string());
            continue;
        }
        let parts = split_fields(trimmed);
        let [qid, group, docno, rel] = parts[..] else {
            return Err(IOError::FormatError(ParseError::InvalidFieldCount {
                line_num,
                expected: FieldCount::Exactly(4),
                found: parts.len(),
            }));
        };
        let rel = parse_grade(rel, line_num, warnings)?;
        records.push(record(line_num, [qid, group, docno], rel));
    }
    if records.is_empty() {
        return Err(IOError::FormatError(ParseError::NoDataLines));
    }
    Ok(records)
}

/// Parses standard qrels relevance judgments file format. A document judged twice in a topic
/// is kept: evaluation refuses it only in a scored topic, and only when a measure reads the
/// ranking ([`Measure::reads_ranking`](crate::metrics::Measure::reads_ranking)).
pub fn parse_trec_qrels<P: AsRef<Path>>(path: P) -> Result<QrelsData, IOError> {
    let file = File::open(path).map_err(IOError::FileReadError)?;
    parse_trec_qrels_from(BufReader::new(file))
}

fn parse_trec_qrels_from(reader: impl BufRead) -> Result<QrelsData, IOError> {
    let mut comments = Vec::new();
    let mut warnings = Vec::new();
    let mut raw_records = read_judgments(
        reader,
        &mut comments,
        &mut warnings,
        |_, [qid, _, docno], rel| RawQrels {
            qid: qid.to_string(),
            docno: docno.to_string(),
            rel,
        },
    )?;

    // Sort by qid first, then docno lexicographically ascending
    raw_records.sort_by(|a, b| {
        match a.qid.cmp(&b.qid) {
            std::cmp::Ordering::Equal => a.docno.cmp(&b.docno),
            other => mt_cmp_order(other),
        }
    });

    // Group into QrelsQuery records in a linear pass
    let mut queries = Vec::new();
    let mut current_query = QrelsQuery {
        qid: raw_records[0].qid.clone(),
        records: vec![QrelsRecord {
            docno: raw_records[0].docno.clone(),
            rel: raw_records[0].rel,
        }],
    };

    for raw in raw_records.into_iter().skip(1) {
        if raw.qid == current_query.qid {
            current_query.records.push(QrelsRecord {
                docno: raw.docno,
                rel: raw.rel,
            });
        } else {
            queries.push(current_query);
            current_query = QrelsQuery {
                qid: raw.qid,
                records: vec![QrelsRecord {
                    docno: raw.docno,
                    rel: raw.rel,
                }],
            };
        }
    }
    queries.push(current_query);

    Ok(QrelsData {
        queries,
        comments,
        warnings,
    })
}

struct RawQrelsJG {
    qid: String,
    jg: String,
    docno: String,
    rel: i64,
    line_num: usize,
}

/// Parses judgment groups qrels format (qid  jg  docno  rel).
pub fn parse_trec_qrels_jg<P: AsRef<Path>>(path: P) -> Result<QrelsJGData, IOError> {
    let file = File::open(path).map_err(IOError::FileReadError)?;
    parse_trec_qrels_jg_from(BufReader::new(file))
}

fn parse_trec_qrels_jg_from(reader: impl BufRead) -> Result<QrelsJGData, IOError> {
    let mut comments = Vec::new();
    let mut warnings = Vec::new();
    let mut raw_records = read_judgments(
        reader,
        &mut comments,
        &mut warnings,
        |line_num, [qid, jg, docno], rel| RawQrelsJG {
            qid: qid.to_string(),
            jg: jg.to_string(),
            docno: docno.to_string(),
            rel,
            line_num,
        },
    )?;

    // Sort by qid first, then jg, then docno lexicographically ascending
    raw_records.sort_by(|a, b| {
        match a.qid.cmp(&b.qid) {
            std::cmp::Ordering::Equal => match a.jg.cmp(&b.jg) {
                std::cmp::Ordering::Equal => a.docno.cmp(&b.docno),
                other => mt_cmp_order(other),
            },
            other => mt_cmp_order(other),
        }
    });

    // Validate duplicate documents under same query and judgment group
    for i in 1..raw_records.len() {
        if raw_records[i].qid == raw_records[i - 1].qid
            && raw_records[i].jg == raw_records[i - 1].jg
            && raw_records[i].docno == raw_records[i - 1].docno
        {
            return Err(IOError::FormatError(ParseError::DuplicateDocument {
                line_num: raw_records[i].line_num,
                qid: raw_records[i].qid.clone(),
                docno: raw_records[i].docno.clone(),
            }));
        }
    }

    // Group into QrelsJGQuery and QrelsJGGroup in a linear pass
    let mut queries: Vec<QrelsJGQuery> = Vec::new();

    for raw in raw_records {
        if let Some(last_q) = queries.last_mut().filter(|q| q.qid == raw.qid) {
            if let Some(last_g) = last_q.groups.last_mut().filter(|g| g.jg == raw.jg) {
                last_g.records.push(QrelsRecord {
                    docno: raw.docno,
                    rel: raw.rel,
                });
            } else {
                last_q.groups.push(QrelsJGGroup {
                    jg: raw.jg,
                    records: vec![QrelsRecord {
                        docno: raw.docno,
                        rel: raw.rel,
                    }],
                });
            }
        } else {
            queries.push(QrelsJGQuery {
                qid: raw.qid,
                groups: vec![QrelsJGGroup {
                    jg: raw.jg,
                    records: vec![QrelsRecord {
                        docno: raw.docno,
                        rel: raw.rel,
                    }],
                }],
            });
        }
    }

    Ok(QrelsJGData {
        queries,
        comments,
        warnings,
    })
}

/// Parses standard results (run) file format. A document retrieved twice for a topic is kept:
/// evaluation refuses it only among the first `max_num_docs_per_topic` documents of a scored
/// topic, and only when a measure reads the ranking
/// ([`Measure::reads_ranking`](crate::metrics::Measure::reads_ranking)).
pub fn parse_trec_run<P: AsRef<Path>>(path: P) -> Result<RunData, IOError> {
    let file = File::open(path).map_err(IOError::FileReadError)?;
    parse_trec_run_from(BufReader::new(file))
}

fn parse_trec_run_from(reader: impl BufRead) -> Result<RunData, IOError> {
    let mut raw_records = Vec::new();
    let mut comments = Vec::new();
    let mut warnings = Vec::new();
    let mut last_run_id = String::new();

    for line in read_lines(reader) {
        let (line_num, line) = line?;
        let trimmed = line.trim_matches(is_field_sep);

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            comments.push(trimmed[1..].to_string());
            continue;
        }

        let parts = split_fields(trimmed);
        if parts.len() < 6 {
            return Err(IOError::FormatError(ParseError::InvalidFieldCount {
                line_num,
                expected: FieldCount::AtLeast(6),
                found: parts.len(),
            }));
        }

        let qid = parts[0].to_string();
        let docno = parts[2].to_string();
        let sim_str = parts[4];
        let run_id = parts[5].to_string();

        let sim = parse_score(sim_str, line_num, &mut warnings)?;

        last_run_id = run_id.clone();

        raw_records.push(RawRun {
            qid,
            docno,
            sim,
            run_id,
        });
    }

    if raw_records.is_empty() {
        return Err(IOError::FormatError(ParseError::NoDataLines));
    }

    // Sort raw records by qid first, then docno lexicographically ascending
    raw_records.sort_by(|a, b| {
        match a.qid.cmp(&b.qid) {
            std::cmp::Ordering::Equal => a.docno.cmp(&b.docno),
            other => mt_cmp_order(other),
        }
    });

    // Group into RunQuery records in a linear pass
    let mut queries = Vec::new();
    let mut current_query = RunQuery {
        qid: raw_records[0].qid.clone(),
        records: vec![RunRecord {
            docno: raw_records[0].docno.clone(),
            sim: raw_records[0].sim,
        }],
    };

    for raw in raw_records.into_iter().skip(1) {
        if raw.qid == current_query.qid {
            current_query.records.push(RunRecord {
                docno: raw.docno,
                sim: raw.sim,
            });
        } else {
            queries.push(current_query);
            current_query = RunQuery {
                qid: raw.qid,
                records: vec![RunRecord {
                    docno: raw.docno,
                    sim: raw.sim,
                }],
            };
        }
    }
    queries.push(current_query);

    Ok(RunData {
        run_id: last_run_id,
        queries,
        comments,
        warnings,
    })
}

#[inline]
fn mt_cmp_order(ord: std::cmp::Ordering) -> std::cmp::Ordering {
    ord
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::{
        parse_trec_qrels_from as qrels, parse_trec_qrels_jg_from as qrels_jg,
        parse_trec_run_from as run,
    };

    fn error<T: std::fmt::Debug>(result: Result<T, IOError>) -> String {
        result.unwrap_err().to_string()
    }

    #[test]
    fn leading_integer_skips_only_c_locale_whitespace() {
        for (value, expected) in [("+2", (2, false)), ("-2x", (-2, true)), ("2 ", (2, true))] {
            assert_eq!(
                read_leading_integer(&format!(" \t\n\x0b\x0c\r{value}")),
                Ok(expected)
            );
        }
        for value in ["\u{a0}2", " + 2", "+"] {
            assert_eq!(
                read_leading_integer(value),
                Err(std::num::IntErrorKind::InvalidDigit)
            );
        }
    }

    #[test]
    fn invalid_line_bytes_name_the_line_and_byte() {
        for (text, message) in [
            (&b"# header\n#\0\n"[..], "Line 2: NUL character"),
            (&b"#\r#\n"[..], "Line 1: CR character without LF"),
            // The byte offset includes the initial BOM.
            (
                &b"\xef\xbb\xbf1 0 d\xe9 1\n"[..],
                "Line 1: invalid UTF-8 at byte 9",
            ),
        ] {
            let expected = format!("Format Error: {message}");
            assert_eq!(error(qrels(text)), expected);
            assert_eq!(error(qrels_jg(text)), expected);
            assert_eq!(error(run(text)), expected);
        }
    }

    #[test]
    fn crlf_and_a_cr_that_ends_the_file_end_lines() {
        for text in [&b"line\r\n"[..], &b"line\r"[..]] {
            assert_eq!(
                read_lines(text).collect::<Result<Vec<_>, _>>().unwrap(),
                [(1, "line".into())]
            );
        }
    }

    #[test]
    fn a_comment_may_hold_bytes_that_are_not_utf8() {
        let lines = read_lines(&b" \t# Ren\xe9\n"[..])
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(lines, [(1, " \t# Ren\u{fffd}".into())]);
    }

    #[test]
    fn a_byte_order_mark_is_removed_only_at_the_start_of_the_file() {
        let lines = read_lines(&b"\xef\xbb\xbf1\n\xef\xbb\xbf2\n"[..])
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(lines, [(1, "1".into()), (2, "\u{feff}2".into())]);
    }

    #[test]
    fn only_c_locale_whitespace_separates_fields() {
        assert_eq!(
            split_fields("1\x0b0\x0cd\u{a0}1\t1"),
            ["1", "0", "d\u{a0}1", "1"]
        );
        let text = "\u{a0}1 0 d 1\u{a0} \n".as_bytes();
        let q = qrels(text).unwrap();
        assert_eq!(q.queries[0].qid, "\u{a0}1");
        assert_eq!(
            q.warnings,
            ["Line 1: value '1\u{a0}' partially read as '1'"]
        );
        assert_eq!(qrels_jg(text).unwrap().queries[0].qid, "\u{a0}1");
        let r = run("\u{a0}1 Q0 d 1 1 r\u{a0} \n".as_bytes()).unwrap();
        assert_eq!(r.queries[0].qid, "\u{a0}1");
        assert_eq!(r.run_id, "r\u{a0}");
    }

    #[test]
    fn blank_lines_warn_only_in_judgments() {
        let text = &b"1 0 a 1\n\n"[..];
        assert_eq!(
            qrels(text).unwrap().warnings,
            ["Line 2: blank line skipped"]
        );
        assert_eq!(
            qrels_jg(text).unwrap().warnings,
            ["Line 2: blank line skipped"]
        );
        assert!(run(&b"1 Q0 a 1 1 r\n\n"[..]).unwrap().warnings.is_empty());
    }

    #[test]
    fn empty_and_comment_only_files_are_distinct_errors() {
        let comments = &b"#\n"[..];
        assert_eq!(error(qrels(comments)), "Format Error: No data lines");
        assert_eq!(error(qrels_jg(comments)), "Format Error: No data lines");
        assert_eq!(error(run(comments)), "Format Error: No data lines");
        assert_eq!(error(run(&b""[..])), "Format Error: File is empty");
    }

    #[test]
    fn field_counts_are_exact_for_judgments_and_minimum_for_runs() {
        for (text, count) in [(&b"1 0 a 1 extra\n"[..], 5), (&b"1 0 a\n"[..], 3)] {
            let expected = format!("Format Error: Line 1: expected 4 fields, found {count}");
            assert_eq!(error(qrels(text)), expected);
            assert_eq!(error(qrels_jg(text)), expected);
        }
        assert_eq!(
            error(run(&b"1 Q0 a 1 2\n"[..])),
            "Format Error: Line 1: expected at least 6 fields, found 5"
        );
        run(&b"1 Q0 a 1 2 r extra\n"[..]).unwrap();
    }

    #[test]
    fn a_grade_reads_as_its_leading_integer_with_a_warning() {
        let text = &b"1 0 a 1.5\n"[..];
        let q = qrels(text).unwrap();
        assert_eq!(q.queries[0].records[0].rel, 1);
        assert_eq!(q.warnings, ["Line 1: value '1.5' partially read as '1'"]);
        let q = qrels_jg(text).unwrap();
        assert_eq!(q.queries[0].groups[0].records[0].rel, 1);
        assert_eq!(q.warnings, ["Line 1: value '1.5' partially read as '1'"]);
        assert_eq!(
            error(qrels(&b"1 0 a .5\n"[..])),
            "Format Error: Line 1: invalid integer for field 'rel': '.5'"
        );
    }

    #[test]
    fn grade_limits_are_inclusive() {
        let edge = &b"1 0 a 127\n1 0 b -127\n"[..];
        qrels(edge).unwrap();
        qrels_jg(edge).unwrap();
        for grade in [-128, 128] {
            let text = format!("1 0 a {grade}\n");
            let expected =
                format!("Format Error: Line 1: relevance level {grade} is outside -127 to 127");
            assert_eq!(error(qrels(text.as_bytes())), expected);
            assert_eq!(error(qrels_jg(text.as_bytes())), expected);
        }
    }

    #[test]
    fn hex_exponent_cancels_a_long_significand_offset() {
        // Four bits per hex digit make both explicit exponents exceed 2^20; each value is still 1.
        let digits = 262_145;
        let zeros = "0".repeat(digits);
        for s in [
            format!("0x0.{zeros}1p{}", 4 * (digits + 1)),
            format!("0x1{zeros}p-{}", 4 * digits),
        ] {
            let (value, len) = strtod_prefix(&s).unwrap();
            assert_eq!((value.to_bits(), len), (1.0f64.to_bits(), s.len()));
        }
    }

    #[test]
    fn hex_exponents_beyond_i64_range_overflow_or_underflow() {
        for (s, expected) in [
            ("0x1p99999999999999999999", f64::INFINITY),
            ("0x1p-99999999999999999999", 0.0),
            ("-0x1p-99999999999999999999", -0.0),
            ("0x0p99999999999999999999", 0.0),
        ] {
            let (value, len) = strtod_prefix(s).unwrap();
            assert_eq!((value.to_bits(), len), (expected.to_bits(), s.len()));
        }
    }

    /// Each case's value and length are those of the C library's `strtod` (macOS 26, arm64).
    #[test]
    fn strtod_prefix_reads_what_strtod_reads() {
        let cases: &[(&str, f64, usize)] = &[
            ("1.5", 1.5, 3),
            ("1.5x", 1.5, 3),
            ("5e", 5.0, 1),
            ("5e+", 5.0, 1),
            ("+.5e1", 5.0, 5),
            (".5", 0.5, 2),
            ("5.", 5.0, 2),
            ("-0", -0.0, 2),
            ("1e400", f64::INFINITY, 5),
            ("-1e400", f64::NEG_INFINITY, 6),
            ("1e-400", 0.0, 6),
            ("inf", f64::INFINITY, 3),
            ("-Infinity", f64::NEG_INFINITY, 9),
            ("infinit", f64::INFINITY, 3),
            ("0x10", 16.0, 4),
            ("0x1p3", 8.0, 5),
            ("-0X1P-2", -0.25, 7),
            ("0x1.8p1", 3.0, 7),
            ("0x.8", 0.5, 4),
            ("0x", 0.0, 1),
            ("0xg", 0.0, 1),
            ("0x1p", 1.0, 3),
            ("0x1p+", 1.0, 3),
            ("0x1.0000000000001p0", 1.0000000000000002, 19),
            // Halfway cases round to even, and a nonzero digit past the mantissa breaks the tie.
            ("0x1.00000000000008p0", 1.0, 20),
            ("0x1.00000000000018p0", 1.0000000000000004, 20),
            ("0x1.000000000000080000000001p0", 1.0000000000000002, 30),
            ("0x1.fffffffffffffp1023", f64::MAX, 22),
            ("0x1.fffffffffffff8p1023", f64::INFINITY, 23),
            ("0x1p-1074", 5e-324, 9),
            ("0x1.8p-1074", 1e-323, 11),
            ("0x1p-1075", 0.0, 9),
        ];
        for &(s, value, len) in cases {
            let (v, n) = strtod_prefix(s).unwrap_or_else(|| panic!("{s} not read"));
            assert_eq!((v.to_bits(), n), (value.to_bits(), len), "{s}: {v}");
        }
    }

    /// Whether the C library's `strtod` sets `ERANGE` and returns ±inf or ±0 (macOS 26, arm64).
    #[test]
    fn out_of_range_matches_strtod_erange() {
        let cases = [
            ("1e400", true),
            ("-1e400", true),
            ("1.7976931348623159e308", true),
            ("1.7976931348623157e308", false),
            ("1e-400", true),
            ("-1e-400", true),
            ("1e-324", true),
            ("3e-324", false),
            ("1e-310", false),
            ("0e-400", false),
            ("0x1p2000", true),
            ("0x1p-1075", true),
            ("0x1p-1074", false),
            ("0x0p-5000", false),
            ("inf", false),
            ("-Infinity", false),
        ];
        for (s, expected) in cases {
            let (value, len) = strtod_prefix(s).unwrap();
            assert_eq!(out_of_range(&s[..len], value), expected, "{s}");
        }
    }

    #[test]
    fn strtod_prefix_reads_no_nan_and_nothing_without_a_number() {
        for s in [
            "nan",
            "-NaN",
            "nan(12ab)",
            "",
            ".",
            "+",
            "-",
            "e5",
            "in",
            "abc",
            "x1",
        ] {
            assert_eq!(strtod_prefix(s), None, "{s}");
        }
    }
}
