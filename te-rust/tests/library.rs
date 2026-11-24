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

#[test]
fn alignment_refuses_grades_outside_the_documented_range() {
    for grade in [128, -128] {
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
            records: judgments("1", &[("a", 128)]).records,
        }],
    };
    assert!(matches!(
        align_query_jg(None, &qrels, "r", 1, usize::MAX, false),
        Err(EvaluationError::RelevanceOutOfBounds { .. })
    ));
}

#[test]
fn judgment_group_alignment_refuses_a_document_judged_twice_in_one_group() {
    let group = |jg: &str, records: &[(&str, i64)]| QrelsJGGroup {
        jg: jg.into(),
        records: judgments("1", records).records,
    };
    let twice = QrelsJGQuery {
        qid: "1".into(),
        groups: vec![group("g", &[("a", 0), ("a", 1)])],
    };
    assert_eq!(
        align_query_jg(None, &twice, "r", 1, usize::MAX, false).unwrap_err(),
        EvaluationError::DuplicateDocument {
            source: DuplicateSource::Qrels,
            qid: "1".into(),
            docno: "a".into(),
        }
    );
    let once_per_group = QrelsJGQuery {
        qid: "1".into(),
        groups: vec![group("g", &[("a", 0)]), group("h", &[("a", 1)])],
    };
    assert!(align_query_jg(None, &once_per_group, "r", 1, usize::MAX, false).is_ok());
}

#[test]
fn alignment_accepts_the_documented_grade_boundaries() {
    for grade in [-127, 127] {
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
        128,
        usize::MAX,
        false,
    )
    .unwrap();
    assert_eq!(state.rel_levels.len(), 128);
}

#[test]
fn alignment_refuses_nan_scores() {
    let run = results("1", &[("b", f64::NAN)]);
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
fn utility_keeps_a_collection_size_above_i64_max() {
    let qrels = qrels(vec![judgments("1", &[("a", 1)])]);
    let run = run(vec![results("1", &[("a", 1.0)])]);
    let utility: Vec<Box<dyn Measure>> = vec![Box::new(
        UtilityMeasure::new(vec![0.0, 0.0, 0.0, 1.0], "").unwrap(),
    )];
    let num_docs_in_coll = usize::MAX;
    let config = EvalConfig {
        num_docs_in_coll,
        ..Default::default()
    };
    let out = evaluate_run(&qrels, &run, &utility, &config, None).unwrap();
    // d = N - 1, the one retrieved document being relevant.
    assert_eq!(
        out.summary_results[0].value,
        MetricValue::Float((num_docs_in_coll - 1) as f64),
        "{num_docs_in_coll}"
    );
}

#[test]
fn all_query_values_adds_the_measures_printed_only_in_the_summary() {
    let qrels = qrels(vec![judgments("1", &[("a", 1)])]);
    let jg = QrelsJGData {
        queries: vec![QrelsJGQuery {
            qid: "1".into(),
            groups: vec![QrelsJGGroup {
                jg: "g".into(),
                records: qrels.queries[0].records.clone(),
            }],
        }],
        comments: vec![],
        warnings: vec![],
    };
    let run = run(vec![results("1", &[("a", 1.0)])]);
    for all_query_values in [false, true] {
        let config = EvalConfig {
            query_flag: true,
            all_query_values,
            ..Default::default()
        };
        for output in [
            evaluate_run(&qrels, &run, &measures(&["num_q"]), &config, None),
            evaluate_run_jg(&jg, &run, &measures(&["num_q"]), &config, None),
        ] {
            let scores = output.unwrap().query_results.remove(0).scores;
            assert_eq!(scores.len(), usize::from(all_query_values));
            if all_query_values {
                assert_eq!(scores[0].1, MetricValue::Integer(1));
            }
        }
    }
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
    let level = -1;
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
