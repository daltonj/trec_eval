use std::collections::{HashMap, HashSet};

use crate::eval::{DuplicateSource, EvaluationError};
use crate::io::parsers::REL_LEVELS;
use crate::io::{QrelsJGQuery, QrelsQuery, QrelsRecord, RunQuery, RunRecord};

pub const RELVALUE_NONPOOL: i64 = -1;

pub const RELVALUE_UNJUDGED: i64 = -2;

/// Aligned evaluation state for a single query topic.
#[derive(Debug, Clone)]
pub struct QueryEvalState {
    /// The unique query topic ID.
    pub qid: String,
    /// The global run label.
    pub run_id: String,

    /// Aligned document list containing relevance judgments at each retrieved rank.
    /// Index matches retrieved rank (0-based).
    /// Value is the relevance level:
    ///   - >= 0: Document was judged and has this relevance grade.
    ///   - -1 (RELVALUE_NONPOOL): Document was retrieved but not judged (not in qrels pool).
    ///   - -2 (RELVALUE_UNJUDGED): Document was in pool but not judged (e.g. infAP).
    pub results_rel_list: Vec<i64>,

    /// Total number of retrieved documents evaluated (after optional truncation and judged-docs filtering).
    pub num_ret: usize,

    /// Total number of relevant documents for this topic in the judgments file.
    /// Specifically, count of documents with relevance judgment >= relevance_level_cutoff.
    pub num_rel: usize,

    /// Total retrieved relevant documents (relevance >= relevance_level_cutoff).
    pub num_rel_ret: usize,

    /// Total retrieved documents not present in the relevance judgments.
    pub num_nonpool: usize,

    /// Total retrieved documents present in judgments but marked unjudged (value < 0).
    pub num_unjudged_in_pool: usize,

    /// Flat array where `rel_levels[g]` stores the total number of documents in qrels with relevance grade `g`.
    /// Sized defensively as `max(max_grade + 1, min(relevance_level_cutoff, 127) + 1)`, where 127 is
    /// the highest level in `REL_LEVELS`.
    pub rel_levels: Vec<usize>,
}

/// Checks the relevance level `rel` of document `docno` in query `qid`.
///
/// # Errors
///
/// Returns [`EvaluationError::RelevanceOutOfBounds`] when `rel` is outside [`REL_LEVELS`].
pub fn check_rel_level(qid: &str, docno: &str, rel: i64) -> Result<(), EvaluationError> {
    if REL_LEVELS.contains(&rel) {
        Ok(())
    } else {
        Err(EvaluationError::RelevanceOutOfBounds {
            qid: qid.to_string(),
            docno: docno.to_string(),
            value: rel,
        })
    }
}

/// Aligns standard run results with qrels judgments for a single query.
///
/// # Errors
///
/// Returns an [`EvaluationError`] when a run score is NaN, when a document repeats within the
/// first `max_docs_per_topic` documents of the ranking or within the judgments, or when a
/// relevance level is outside [`REL_LEVELS`].
pub fn align_query(
    run_query: Option<&RunQuery>,
    qrels_query: &QrelsQuery,
    run_id: &str,
    relevance_level_cutoff: i64,
    max_docs_per_topic: usize,
    judged_docs_only_flag: bool,
) -> Result<QueryEvalState, EvaluationError> {
    align_query_impl(
        run_query,
        qrels_query,
        run_id,
        relevance_level_cutoff,
        max_docs_per_topic,
        judged_docs_only_flag,
        true,
    )
}

/// As [`align_query`], refusing repeated documents only when `check_duplicates` is set.
pub(crate) fn align_query_impl(
    run_query: Option<&RunQuery>,
    qrels_query: &QrelsQuery,
    run_id: &str,
    relevance_level_cutoff: i64,
    max_docs_per_topic: usize,
    judged_docs_only_flag: bool,
    check_duplicates: bool,
) -> Result<QueryEvalState, EvaluationError> {
    let doc_infos = rank_documents(
        run_query,
        &qrels_query.qid,
        max_docs_per_topic,
        check_duplicates,
    )?;
    align_ranked_query(
        &doc_infos,
        &qrels_query.qid,
        &qrels_query.records,
        run_id,
        relevance_level_cutoff,
        judged_docs_only_flag,
        check_duplicates,
    )
}

fn rank_documents<'a>(
    run_query: Option<&'a RunQuery>,
    qid: &str,
    max_docs_per_topic: usize,
    check_duplicates: bool,
) -> Result<Vec<&'a RunRecord>, EvaluationError> {
    let mut doc_infos: Vec<_> = run_query.into_iter().flat_map(|q| &q.records).collect();
    if let Some(doc) = doc_infos.iter().find(|doc| doc.sim.is_nan()) {
        return Err(EvaluationError::NanScore {
            qid: qid.to_string(),
            docno: doc.docno.clone(),
        });
    }

    // Sort run records by sim descending, and docno descending lexicographically
    doc_infos.sort_by(|a, b| {
        // Sort sim descending
        match b.sim.partial_cmp(&a.sim) {
            Some(std::cmp::Ordering::Equal) => {
                // Tie breaker: docno descending lexicographically
                b.docno.cmp(&a.docno)
            }
            Some(other) => other,
            None => unreachable!("NaN scores are refused above"),
        }
    });

    // Truncate to max_docs_per_topic
    if doc_infos.len() > max_docs_per_topic {
        doc_infos.truncate(max_docs_per_topic);
    }

    if check_duplicates {
        let mut seen = HashSet::with_capacity(doc_infos.len());
        for doc in &doc_infos {
            if !seen.insert(&doc.docno) {
                return Err(EvaluationError::DuplicateDocument {
                    source: DuplicateSource::Run,
                    qid: qid.to_string(),
                    docno: doc.docno.clone(),
                });
            }
        }
    }

    Ok(doc_infos)
}

fn align_ranked_query(
    doc_infos: &[&RunRecord],
    qid: &str,
    records: &[QrelsRecord],
    run_id: &str,
    relevance_level_cutoff: i64,
    judged_docs_only_flag: bool,
    check_duplicates: bool,
) -> Result<QueryEvalState, EvaluationError> {
    let mut qrels_map = HashMap::with_capacity(records.len());
    for record in records {
        if qrels_map.insert(&record.docno, record.rel).is_some() && check_duplicates {
            return Err(EvaluationError::DuplicateDocument {
                source: DuplicateSource::Qrels,
                qid: qid.to_string(),
                docno: record.docno.clone(),
            });
        }
    }

    // Calculate max relevance grade and populate rel_levels counts
    let mut max_rel_grade = 0;
    for record in records {
        check_rel_level(qid, &record.docno, record.rel)?;
        if record.rel > max_rel_grade {
            max_rel_grade = record.rel;
        }
    }

    let rel_levels_size = std::cmp::max(max_rel_grade + 1, relevance_level_cutoff.min(*REL_LEVELS.end()) + 1) as usize;
    let mut rel_levels = vec![0; rel_levels_size];
    let mut num_rel = 0;

    for record in records {
        if record.rel >= 0 {
            rel_levels[record.rel as usize] += 1;
            if record.rel >= relevance_level_cutoff {
                num_rel += 1;
            }
        }
    }

    let raw_num_ret = doc_infos.len();

    // Construct results_rel_list and count metrics
    let mut results_rel_list = Vec::with_capacity(raw_num_ret);
    let mut num_rel_ret = 0;
    let mut num_nonpool = 0;
    let mut num_unjudged_in_pool = 0;

    for doc in doc_infos {
        match qrels_map.get(&doc.docno) {
            Some(&rel) => {
                if rel < 0 {
                    results_rel_list.push(RELVALUE_UNJUDGED);
                    num_unjudged_in_pool += 1;
                } else {
                    results_rel_list.push(rel);
                    if rel >= relevance_level_cutoff {
                        num_rel_ret += 1;
                    }
                }
            }
            None => {
                results_rel_list.push(RELVALUE_NONPOOL);
                num_nonpool += 1;
            }
        }
    }

    // If judged_docs_only_flag is set, throw out all unjudged docs
    if judged_docs_only_flag {
        results_rel_list.retain(|&rel| rel >= 0);
        // Recalculate num_rel_ret from the filtered set
        num_rel_ret = results_rel_list
            .iter()
            .filter(|&&rel| rel >= relevance_level_cutoff)
            .count();
        num_nonpool = 0;
        num_unjudged_in_pool = 0;
    }

    let num_ret = results_rel_list.len();

    Ok(QueryEvalState {
        qid: qid.to_string(),
        run_id: run_id.to_string(),
        results_rel_list,
        num_ret,
        num_rel,
        num_rel_ret,
        num_nonpool,
        num_unjudged_in_pool,
        rel_levels,
    })
}

/// Aligns standard run results with judgment group relevance judgments for a single query.
///
/// # Errors
///
/// As [`align_query`], where a document repeats within the judgments when it repeats within
/// one group.
pub fn align_query_jg(
    run_query: Option<&RunQuery>,
    qrels_jg_query: &QrelsJGQuery,
    run_id: &str,
    relevance_level_cutoff: i64,
    max_docs_per_topic: usize,
    judged_docs_only_flag: bool,
) -> Result<Vec<QueryEvalState>, EvaluationError> {
    align_query_jg_impl(
        run_query,
        qrels_jg_query,
        run_id,
        relevance_level_cutoff,
        max_docs_per_topic,
        judged_docs_only_flag,
        true,
    )
}

/// As [`align_query_jg`], refusing repeated documents only when `check_duplicates` is set.
pub(crate) fn align_query_jg_impl(
    run_query: Option<&RunQuery>,
    qrels_jg_query: &QrelsJGQuery,
    run_id: &str,
    relevance_level_cutoff: i64,
    max_docs_per_topic: usize,
    judged_docs_only_flag: bool,
    check_duplicates: bool,
) -> Result<Vec<QueryEvalState>, EvaluationError> {
    let doc_infos = rank_documents(
        run_query,
        &qrels_jg_query.qid,
        max_docs_per_topic,
        check_duplicates,
    )?;
    qrels_jg_query
        .groups
        .iter()
        .map(|group| {
            align_ranked_query(
                &doc_infos,
                &qrels_jg_query.qid,
                &group.records,
                run_id,
                relevance_level_cutoff,
                judged_docs_only_flag,
                check_duplicates,
            )
        })
        .collect()
}

