//! The library API on data that a caller builds in memory, which no file reader has checked.

use te_rust::eval::{
    align_query, align_query_jg, evaluate_run, evaluate_run_jg, DuplicateSource, EvaluationError,
};
use te_rust::io::{
    QrelsData, QrelsJGData, QrelsJGGroup, QrelsJGQuery, QrelsQuery, QrelsRecord, RunData, RunQuery,
    RunRecord,
};
use te_rust::metrics::registry::resolve_measures;
use te_rust::metrics::utility::UtilityMeasure;
use te_rust::metrics::{EvalConfig, Measure, MetricValue};

/// The judgments `(docno, rel)` of topic `qid`.
fn judgments(qid: &str, records: &[(&str, i64)]) -> QrelsQuery {
    QrelsQuery {
        qid: qid.into(),
        records: records
            .iter()
            .map(|&(docno, rel)| QrelsRecord {
                docno: docno.into(),
                rel,
            })
            .collect(),
    }
}

/// The results `(docno, sim)` of topic `qid`.
fn results(qid: &str, records: &[(&str, f64)]) -> RunQuery {
    RunQuery {
        qid: qid.into(),
        records: records
            .iter()
            .map(|&(docno, sim)| RunRecord {
                docno: docno.into(),
                sim,
            })
            .collect(),
    }
}

fn qrels(queries: Vec<QrelsQuery>) -> QrelsData {
    QrelsData {
        queries,
        comments: vec![],
        warnings: vec![],
    }
}

fn run(queries: Vec<RunQuery>) -> RunData {
    RunData {
        run_id: "r".into(),
        queries,
        comments: vec![],
        warnings: vec![],
    }
}

fn measures(requests: &[&str]) -> Vec<Box<dyn Measure>> {
    let requests: Vec<String> = requests.iter().map(|r| r.to_string()).collect();
    resolve_measures(&requests).unwrap().measures
}

fn repeated(source: DuplicateSource, qid: &str, docno: &str) -> EvaluationError {
    EvaluationError::DuplicateDocument {
        source,
        qid: qid.into(),
        docno: docno.into(),
    }
}

#[test]
fn a_repeated_document_in_a_scored_ranking_is_refused() {
    let repeat = run(vec![results("1", &[("a", 2.0), ("a", 1.0)])]);
    let config = EvalConfig::default();
    let qrels = qrels(vec![judgments("1", &[("a", 1)])]);
    let result = evaluate_run(
        &qrels,
        &repeat,
        &measures(&["map", "recall.2"]),
        &config,
        None,
    );
    assert_eq!(
        result.unwrap_err(),
        repeated(DuplicateSource::Run, "1", "a")
    );
    let qrels_jg = QrelsJGData {
        queries: vec![QrelsJGQuery {
            qid: "1".into(),
            groups: vec![QrelsJGGroup {
                jg: "g".into(),
                records: judgments("1", &[("a", 1)]).records,
            }],
        }],
        comments: vec![],
        warnings: vec![],
    };
    let result = evaluate_run_jg(&qrels_jg, &repeat, &measures(&["map_avgjg"]), &config, None);
    assert_eq!(
        result.unwrap_err(),
        repeated(DuplicateSource::Run, "1", "a")
    );
}

#[test]
fn duplicates_are_ignored_after_truncation_or_without_ranking_measures() {
    let repeat = run(vec![results("1", &[("a", 2.0), ("a", 1.0)])]);
    let qrels = qrels(vec![judgments("1", &[("a", 1)])]);
    for (request, max_docs) in [("map", 1), ("num_q", usize::MAX), ("runid", usize::MAX)] {
        let config = EvalConfig {
            max_num_docs_per_topic: max_docs,
            ..Default::default()
        };
        assert!(
            evaluate_run(&qrels, &repeat, &measures(&[request]), &config, None).is_ok(),
            "{request}"
        );
    }
}

#[test]
fn a_qrels_topic_missing_from_the_run_is_checked_only_with_complete_averaging() {
    let qrels = qrels(vec![
        judgments("1", &[("a", 1)]),
        judgments("2", &[("b", 1), ("b", 1)]),
    ]);
    let run = run(vec![results("1", &[("a", 2.0)])]);
    let map = measures(&["map"]);
    assert!(evaluate_run(&qrels, &run, &map, &EvalConfig::default(), None).is_ok());
    let config = EvalConfig {
        average_complete_flag: true,
        ..Default::default()
    };
    let result = evaluate_run(&qrels, &run, &map, &config, None);
    assert_eq!(
        result.unwrap_err(),
        repeated(DuplicateSource::Qrels, "2", "b")
    );
}

#[test]
fn alignment_refuses_grades_outside_the_documented_range() {
    for grade in [i64::MAX, i64::MIN, 128, -128] {
        let qrels = judgments("1", &[("a", grade)]);
        assert!(
            matches!(
                align_query(None, &qrels, "r", 1, usize::MAX, false),
                Err(EvaluationError::RelevanceOutOfBounds { .. })
            ),
            "{grade}"
        );
    }
    let qrels = QrelsJGQuery {
        qid: "1".into(),
        groups: vec![QrelsJGGroup {
            jg: "g".into(),
            records: judgments("1", &[("a", i64::MAX)]).records,
        }],
    };
    assert!(matches!(
        align_query_jg(None, &qrels, "r", 1, usize::MAX, false),
        Err(EvaluationError::RelevanceOutOfBounds { .. })
    ));
}

#[test]
fn alignment_accepts_the_documented_grade_boundaries() {
    for grade in [-127, 0, 127] {
        let state = align_query(
            None,
            &judgments("1", &[("a", grade)]),
            "r",
            1,
            usize::MAX,
            false,
        )
        .unwrap();
        assert_eq!(state.num_rel, usize::from(grade >= 1));
        if grade >= 0 {
            assert_eq!(state.rel_levels[grade as usize], 1);
        }
    }
}

#[test]
fn grade_counts_stop_at_grade_127() {
    let state = align_query(
        None,
        &judgments("1", &[("a", 1)]),
        "r",
        1_000_000,
        usize::MAX,
        false,
    )
    .unwrap();
    assert_eq!(state.rel_levels.len(), 128);
}

#[test]
fn alignment_refuses_nan_scores() {
    let run = results("1", &[("a", 1.0), ("b", f64::NAN)]);
    let result = align_query(
        Some(&run),
        &judgments("1", &[("a", 1)]),
        "r",
        1,
        usize::MAX,
        false,
    );
    assert_eq!(
        result.unwrap_err(),
        EvaluationError::NanScore {
            qid: "1".into(),
            docno: "b".into()
        }
    );
}

#[test]
#[cfg(target_pointer_width = "64")]
fn utility_uses_i64_max_for_a_larger_collection() {
    let qrels = qrels(vec![judgments("1", &[("a", 1), ("b", 0)])]);
    let run = run(vec![results("1", &[("a", 3.0), ("x", 2.0), ("y", 1.0)])]);
    let utility: Vec<Box<dyn Measure>> = vec![Box::new(
        UtilityMeasure::new(vec![0.0, 0.0, 0.0, 1.0], "").unwrap(),
    )];
    for num_docs_in_coll in [i64::MAX as usize + 1, usize::MAX] {
        let config = EvalConfig {
            num_docs_in_coll,
            ..Default::default()
        };
        let out = evaluate_run(&qrels, &run, &utility, &config, None).unwrap();
        assert_eq!(
            out.summary_results[0].value,
            MetricValue::Float(i64::MAX as f64),
            "{num_docs_in_coll}"
        );
    }
}

#[test]
fn all_query_values_adds_the_measures_printed_only_in_the_summary() {
    let qrels = qrels(vec![judgments("1", &[("a", 1), ("b", 0)])]);
    let run = run(vec![results("1", &[("b", 2.0), ("a", 1.0)])]);
    let requested = measures(&["gm_map", "num_q", "runid", "map"]);
    let per_topic = |config: EvalConfig| {
        let out = evaluate_run(&qrels, &run, &requested, &config, None).unwrap();
        let mut scores: Vec<(String, MetricValue)> = out.query_results[0]
            .scores
            .iter()
            .map(|(name, value, _)| (name.clone(), value.clone()))
            .collect();
        scores.sort_by(|a, b| a.0.cmp(&b.0));
        scores
    };
    let printed = EvalConfig {
        query_flag: true,
        ..Default::default()
    };
    assert_eq!(
        per_topic(printed),
        [("map".into(), MetricValue::Float(0.5))]
    );
    let all = EvalConfig {
        query_flag: true,
        all_query_values: true,
        ..Default::default()
    };
    assert_eq!(
        per_topic(all),
        [
            ("gm_map".into(), MetricValue::Float(0.5_f64.ln())),
            ("map".into(), MetricValue::Float(0.5)),
            ("num_q".into(), MetricValue::Integer(1)),
            ("runid".into(), MetricValue::Str("r".into())),
        ]
    );
}

#[test]
fn a_negative_relevance_level_is_refused_even_without_topics() {
    let qrels = qrels(vec![]);
    let qrels_jg = QrelsJGData {
        queries: vec![],
        comments: vec![],
        warnings: vec![],
    };
    let run = run(vec![]);
    for level in [-1, i64::MIN] {
        let config = EvalConfig {
            relevance_level: level,
            ..Default::default()
        };
        for result in [
            evaluate_run(&qrels, &run, &[], &config, None),
            evaluate_run_jg(&qrels_jg, &run, &[], &config, None),
        ] {
            assert_eq!(
                result.unwrap_err().to_string(),
                format!("Unusable relevance level '{level}'")
            );
        }
    }
}
