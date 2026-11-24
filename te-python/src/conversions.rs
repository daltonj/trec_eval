use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use pyo3::exceptions::{PyFileNotFoundError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyIterator, PyLong, PySequence, PyString};
use te_rust::eval::check_rel_level;
use te_rust::io::parsers::REL_LEVELS;
use te_rust::io::{parse_trec_qrels, parse_trec_run, QrelsData, QrelsQuery, QrelsRecord, RunData, RunQuery, RunRecord};
use crate::errors::{evaluation_error, warn_all};

/// Extract a file path string from a Python object if it is a `str` or `os.PathLike`.
fn extract_path_string(obj: &Bound<'_, PyAny>) -> Option<String> {
    if let Ok(py_str) = obj.downcast::<PyString>() {
        return Some(py_str.to_string_lossy().into_owned());
    }
    if obj.hasattr("__fspath__").unwrap_or(false) {
        if let Ok(res) = obj.call_method0("__fspath__") {
            if let Ok(py_str) = res.downcast::<PyString>() {
                return Some(py_str.to_string_lossy().into_owned());
            }
        }
    }
    None
}

/// The file that an input was read from: its name as given, for messages, and its absolute path,
/// which still names the file after the working directory changes.
pub struct InputFile {
    pub name: String,
    pub path: PathBuf,
}

impl InputFile {
    fn new(name: String) -> Self {
        let path = std::path::absolute(&name).unwrap_or_else(|_| PathBuf::from(&name));
        Self { name, path }
    }
}

/// Options for mapping DataFrame column names.
#[derive(Debug, Default, Clone)]
pub struct ColumnMapping {
    pub qid: Option<String>,
    pub docno: Option<String>,
    pub score_or_rel: Option<String>,
}

fn find_column_name(
    df: &Bound<'_, PyAny>,
    specified: Option<&str>,
    candidates: &[&str],
    field_label: &str,
) -> PyResult<String> {
    if let Some(col) = specified {
        return Ok(col.to_string());
    }
    if let Ok(cols_obj) = df.getattr("columns") {
        let cols: Vec<String> = if let Ok(iter) = cols_obj.iter() {
            iter.filter_map(|x| x.ok()?.extract::<String>().ok()).collect()
        } else {
            Vec::new()
        };
        for &candidate in candidates {
            if let Some(found) = cols.iter().find(|c| c.eq_ignore_ascii_case(candidate)) {
                return Ok(found.clone());
            }
        }
        return Err(PyValueError::new_err(format!(
            "Could not automatically detect {} column in DataFrame. Available columns: {:?}. Specify column name explicitly.",
            field_label, cols
        )));
    }
    Err(PyTypeError::new_err("Object is not a DataFrame (missing .columns)"))
}

/// Ingest relevance judgments (qrels) from a Python object (file path, dict, DataFrame, or iterable),
/// with the file they were read from, if any.
pub fn ingest_qrels(
    obj: &Bound<'_, PyAny>,
    mapping: Option<&ColumnMapping>,
) -> PyResult<(QrelsData, Option<InputFile>)> {
    // 1. File path / PathLike
    if let Some(path_str) = extract_path_string(obj) {
        let path = Path::new(&path_str);
        if path.is_file() {
            let qrels = parse_trec_qrels(path).map_err(|e| {
                PyValueError::new_err(format!("Failed to parse qrels file '{}': {}", path_str, e))
            })?;
            let warnings = qrels.warnings.iter().map(|w| format!("qrels file '{}': {}", path_str, w));
            warn_all(obj.py(), warnings)?;
            return Ok((qrels, Some(InputFile::new(path_str))));
        }
        return Err(PyFileNotFoundError::new_err(format!(
            "Qrels file not found: '{}'",
            path_str
        )));
    }
    ingest_qrels_value(obj, mapping).map(|qrels| (qrels, None))
}

/// Ingest relevance judgments (qrels) from a Python object that is not a file path.
fn ingest_qrels_value(obj: &Bound<'_, PyAny>, mapping: Option<&ColumnMapping>) -> PyResult<QrelsData> {
    // 2. Nested Dictionary: Dict[str, Dict[str, Union[int, float]]]
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut grouped: BTreeMap<String, BTreeMap<String, i64>> = BTreeMap::new();
        for (qid_obj, docs_obj) in dict.iter() {
            let qid = qid_obj.extract::<String>().map_err(|_| {
                PyTypeError::new_err("Expected query_id to be a string in qrels dict")
            })?;
            let docs_dict = docs_obj.downcast::<PyDict>().map_err(|_| {
                PyTypeError::new_err(format!(
                    "Expected query '{}' value to be a Dict[doc_id, relevance] in qrels",
                    qid
                ))
            })?;

            let query_map = grouped.entry(qid.clone()).or_default();
            for (doc_obj, rel_obj) in docs_dict.iter() {
                let docno = doc_obj.extract::<String>().map_err(|_| {
                    PyTypeError::new_err("Expected doc_id to be a string in qrels dict")
                })?;
                let rel = extract_grade(&rel_obj, &qid, &docno).map_err(|e| {
                    if e.is_instance_of::<PyValueError>(obj.py()) {
                        e
                    } else {
                        PyTypeError::new_err("Expected relevance judgment to be an integer in qrels dict")
                    }
                })?;
                check_rel_level(&qid, &docno, rel).map_err(evaluation_error)?;
                query_map.insert(docno, rel);
            }
        }

        let queries = grouped
            .into_iter()
            .map(|(qid, doc_map)| QrelsQuery {
                qid,
                records: doc_map
                    .into_iter()
                    .map(|(docno, rel)| QrelsRecord { docno, rel })
                    .collect(),
            })
            .collect();

        return Ok(QrelsData {
            queries,
            comments: Vec::new(),
            warnings: Vec::new(),
        });
    }

    // 3. DataFrame (Pandas / Polars) with .columns
    if obj.hasattr("columns").unwrap_or(false) {
        let default_mapping = ColumnMapping::default();
        let map = mapping.unwrap_or(&default_mapping);

        let qid_col = find_column_name(
            obj,
            map.qid.as_deref(),
            &["query_id", "qid", "query", "topic", "topic_id"],
            "query_id",
        )?;
        let doc_col = find_column_name(
            obj,
            map.docno.as_deref(),
            &["doc_id", "docno", "doc", "document"],
            "doc_id",
        )?;
        let rel_col = find_column_name(
            obj,
            map.score_or_rel.as_deref(),
            &["relevance", "rel", "grade", "judgment", "label"],
            "relevance",
        )?;

        let qid_series = obj.get_item(&qid_col)?;
        let doc_series = obj.get_item(&doc_col)?;
        let rel_series = obj.get_item(&rel_col)?;

        let triples = qid_series.iter()?.zip(doc_series.iter()?).zip(rel_series.iter()?);
        return qrels_from_triples(
            triples.map(|((qid, docno), rel)| {
                let (qid, docno): (String, String) = (qid?.extract()?, docno?.extract()?);
                let rel = extract_grade(&rel?, &qid, &docno)?;
                Ok((qid, docno, rel))
            }),
        );
    }

    // 4. Iterable of 3-tuples or objects with (query_id, doc_id, relevance)
    if let Ok(iter) = obj.iter() {
        return qrels_from_triples(iter.map(|item| extract_qrel_item(&item?)));
    }

    Err(PyTypeError::new_err(
        "Invalid qrels input: expected file path, nested dict (Dict[str, Dict[str, int]]), DataFrame, or iterable of (qid, doc_id, rel) triples"
    ))
}


/// Reads a grade as an `i64`. An integer too large for one, an `int` or another integer such as a
/// NumPy integer, is refused with the `ValueError` that `check_rel_level` gives for a grade outside
/// the range that te-rust reads. Another integer is read through `__index__` once.
fn extract_grade(obj: &Bound<'_, PyAny>, qid: &str, docno: &str) -> PyResult<i64> {
    let value = if obj.is_instance_of::<PyLong>() {
        obj.clone()
    } else {
        obj.py().import_bound("operator")?.call_method1("index", (obj,))?
    };
    value.extract::<i64>().map_err(|_| {
        PyValueError::new_err(format!(
            "relevance level {} for document '{}' in query '{}' is outside {} to {}",
            value,
            docno,
            qid,
            REL_LEVELS.start(),
            REL_LEVELS.end()
        ))
    })
}

/// Groups `(qid, docno, rel)` triples by topic, keeping repeated documents for evaluation to
/// refuse, and refuses a grade outside the range that te-rust reads.
fn qrels_from_triples(
    triples: impl Iterator<Item = PyResult<(String, String, i64)>>,
) -> PyResult<QrelsData> {
    let mut grouped: BTreeMap<String, Vec<QrelsRecord>> = BTreeMap::new();
    for triple in triples {
        let (qid, docno, rel) = triple?;
        check_rel_level(&qid, &docno, rel).map_err(evaluation_error)?;
        grouped
            .entry(qid)
            .or_default()
            .push(QrelsRecord { docno, rel });
    }

    let queries = grouped
        .into_iter()
        .map(|(qid, records)| QrelsQuery { qid, records })
        .collect();

    Ok(QrelsData {
        queries,
        comments: Vec::new(),
        warnings: Vec::new(),
    })
}

fn extract_qrel_item(item: &Bound<'_, PyAny>) -> PyResult<(String, String, i64)> {
    if let Ok(seq) = item.downcast::<PySequence>() {
        let len = seq.len()?;
        if len >= 3 {
            let qid = seq.get_item(0)?.extract::<String>()?;
            let docno = seq.get_item(1)?.extract::<String>()?;
            let rel = extract_grade(&seq.get_item(2)?, &qid, &docno)?;
            return Ok((qid, docno, rel));
        }
    }

    // Check named attributes (e.g. ScoredDoc / Qrel / Dataclasses)
    let qid = if let Ok(v) = item.getattr("query_id") {
        v.extract::<String>()?
    } else if let Ok(v) = item.getattr("qid") {
        v.extract::<String>()?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract query_id from qrel item (must be 3-tuple or have .query_id / .qid attribute)"
        ));
    };

    let docno = if let Ok(v) = item.getattr("doc_id") {
        v.extract::<String>()?
    } else if let Ok(v) = item.getattr("docno") {
        v.extract::<String>()?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract doc_id from qrel item (must be 3-tuple or have .doc_id / .docno attribute)"
        ));
    };

    let rel = if let Ok(v) = item.getattr("relevance") {
        extract_grade(&v, &qid, &docno)?
    } else if let Ok(v) = item.getattr("rel") {
        extract_grade(&v, &qid, &docno)?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract relevance from qrel item (must be 3-tuple or have .relevance / .rel attribute)"
        ));
    };

    Ok((qid, docno, rel))
}

/// Ingest run results from a Python object (file path, dict, DataFrame, or iterable), with the file
/// they were read from, if any.
pub fn ingest_run(
    obj: &Bound<'_, PyAny>,
    mapping: Option<&ColumnMapping>,
) -> PyResult<(RunData, Option<InputFile>)> {
    // 1. File path / PathLike
    if let Some(path_str) = extract_path_string(obj) {
        let path = Path::new(&path_str);
        if path.is_file() {
            let run = parse_trec_run(path).map_err(|e| {
                PyValueError::new_err(format!("Failed to parse run file '{}': {}", path_str, e))
            })?;
            let warnings = run.warnings.iter().map(|w| format!("run file '{}': {}", path_str, w));
            warn_all(obj.py(), warnings)?;
            return Ok((run, Some(InputFile::new(path_str))));
        }
        return Err(PyFileNotFoundError::new_err(format!(
            "Run file not found: '{}'",
            path_str
        )));
    }
    ingest_run_value(obj, mapping).map(|run| (run, None))
}

/// Ingest run results from a Python object that is not a file path.
fn ingest_run_value(obj: &Bound<'_, PyAny>, mapping: Option<&ColumnMapping>) -> PyResult<RunData> {
    // 2. Nested Dictionary: Dict[str, Dict[str, float]]
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let mut grouped: BTreeMap<String, Vec<RunRecord>> = BTreeMap::new();
        for (qid_obj, docs_obj) in dict.iter() {
            let qid = qid_obj.extract::<String>().map_err(|_| {
                PyTypeError::new_err("Expected query_id to be a string in run dict")
            })?;
            let docs_dict = docs_obj.downcast::<PyDict>().map_err(|_| {
                PyTypeError::new_err(format!(
                    "Expected query '{}' value to be a Dict[doc_id, score] in run",
                    qid
                ))
            })?;

            let records = grouped.entry(qid).or_default();
            for (doc_obj, sim_obj) in docs_dict.iter() {
                let docno = doc_obj.extract::<String>().map_err(|_| {
                    PyTypeError::new_err("Expected doc_id to be a string in run dict")
                })?;
                let sim = sim_obj.extract::<f64>().map_err(|_| {
                    PyTypeError::new_err("Expected score to be a float in run dict")
                })?;
                records.push(RunRecord { docno, sim });
            }
        }

        let queries = grouped
            .into_iter()
            .map(|(qid, records)| RunQuery { qid, records })
            .collect();

        return Ok(RunData {
            run_id: "trec_eval".to_string(),
            queries,
            comments: Vec::new(),
            warnings: Vec::new(),
        });
    }

    // 3. DataFrame (Pandas / Polars) with .columns
    if obj.hasattr("columns").unwrap_or(false) {
        let default_mapping = ColumnMapping::default();
        let map = mapping.unwrap_or(&default_mapping);

        let qid_col = find_column_name(
            obj,
            map.qid.as_deref(),
            &["query_id", "qid", "query", "topic", "topic_id"],
            "query_id",
        )?;
        let doc_col = find_column_name(
            obj,
            map.docno.as_deref(),
            &["doc_id", "docno", "doc", "document"],
            "doc_id",
        )?;
        let score_col = find_column_name(
            obj,
            map.score_or_rel.as_deref(),
            &["score", "sim", "similarity", "rank_score"],
            "score",
        )?;

        let qid_series = obj.get_item(&qid_col)?;
        let doc_series = obj.get_item(&doc_col)?;
        let score_series = obj.get_item(&score_col)?;

        let mut qid_iter = qid_series.iter()?;
        let mut doc_iter = doc_series.iter()?;
        let mut score_iter = score_series.iter()?;

        let mut grouped: BTreeMap<String, Vec<RunRecord>> = BTreeMap::new();
        while let (Some(q_res), Some(d_res), Some(s_res)) = (qid_iter.next(), doc_iter.next(), score_iter.next()) {
            let qid = q_res?.extract::<String>()?;
            let docno = d_res?.extract::<String>()?;
            let sim = s_res?.extract::<f64>()?;
            grouped.entry(qid).or_default().push(RunRecord { docno, sim });
        }

        let queries = grouped
            .into_iter()
            .map(|(qid, records)| RunQuery { qid, records })
            .collect();

        return Ok(RunData {
            run_id: "trec_eval".to_string(),
            queries,
            comments: Vec::new(),
            warnings: Vec::new(),
        });
    }

    // 4. Iterable of 3-tuples or objects with (query_id, doc_id, score)
    if let Ok(iter) = obj.iter() {
        return ingest_run_from_iter(iter);
    }

    Err(PyTypeError::new_err(
        "Invalid run input: expected file path, nested dict (Dict[str, Dict[str, float]]), DataFrame, or iterable of (qid, doc_id, score) triples"
    ))
}

/// Ingest run results from 1D contiguous arrays/sequences of (query_ids, doc_ids, scores).
pub fn ingest_run_arrays(
    query_ids: &Bound<'_, PyAny>,
    doc_ids: &Bound<'_, PyAny>,
    scores: &Bound<'_, PyAny>,
) -> PyResult<RunData> {
    let mut grouped: BTreeMap<String, Vec<RunRecord>> = BTreeMap::new();

    let mut qid_iter = query_ids.iter()?;
    let mut doc_iter = doc_ids.iter()?;
    let mut score_iter = scores.iter()?;

    while let (Some(q_res), Some(d_res), Some(s_res)) = (qid_iter.next(), doc_iter.next(), score_iter.next()) {
        let qid = q_res?.extract::<String>()?;
        let docno = d_res?.extract::<String>()?;
        let sim = s_res?.extract::<f64>()?;
        grouped.entry(qid).or_default().push(RunRecord { docno, sim });
    }

    let queries = grouped
        .into_iter()
        .map(|(qid, records)| RunQuery { qid, records })
        .collect();

    Ok(RunData {
        run_id: "trec_eval".to_string(),
        queries,
        comments: Vec::new(),
        warnings: Vec::new(),
    })
}


fn ingest_run_from_iter(iter: Bound<'_, PyIterator>) -> PyResult<RunData> {
    let mut grouped: BTreeMap<String, Vec<RunRecord>> = BTreeMap::new();

    for item_res in iter {
        let item = item_res?;
        let (qid, docno, sim) = extract_run_item(&item)?;
        grouped.entry(qid).or_default().push(RunRecord { docno, sim });
    }

    let queries = grouped
        .into_iter()
        .map(|(qid, records)| RunQuery { qid, records })
        .collect();

    Ok(RunData {
        run_id: "trec_eval".to_string(),
        queries,
        comments: Vec::new(),
        warnings: Vec::new(),
    })
}

fn extract_run_item(item: &Bound<'_, PyAny>) -> PyResult<(String, String, f64)> {
    if let Ok(seq) = item.downcast::<PySequence>() {
        let len = seq.len()?;
        if len >= 3 {
            let qid = seq.get_item(0)?.extract::<String>()?;
            let docno = seq.get_item(1)?.extract::<String>()?;
            let sim = seq.get_item(2)?.extract::<f64>()?;
            return Ok((qid, docno, sim));
        }
    }

    let qid = if let Ok(v) = item.getattr("query_id") {
        v.extract::<String>()?
    } else if let Ok(v) = item.getattr("qid") {
        v.extract::<String>()?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract query_id from run item (must be 3-tuple or have .query_id / .qid attribute)"
        ));
    };

    let docno = if let Ok(v) = item.getattr("doc_id") {
        v.extract::<String>()?
    } else if let Ok(v) = item.getattr("docno") {
        v.extract::<String>()?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract doc_id from run item (must be 3-tuple or have .doc_id / .docno attribute)"
        ));
    };

    let sim = if let Ok(v) = item.getattr("score") {
        v.extract::<f64>()?
    } else if let Ok(v) = item.getattr("sim") {
        v.extract::<f64>()?
    } else {
        return Err(PyTypeError::new_err(
            "Could not extract score from run item (must be 3-tuple or have .score / .sim attribute)"
        ));
    };

    Ok((qid, docno, sim))
}
