use te_rust::eval::{evaluate_run, evaluate_run_jg, EvaluationError, ScoredDuplicate};
use te_rust::io::{
    QrelsData, QrelsJGData, QrelsJGGroup, QrelsJGQuery, QrelsQuery, QrelsRecord, RunData, RunQuery,
    RunRecord,
};
use te_rust::metrics::{registry::resolve_measures, EvalConfig};

fn qrels() -> QrelsData {
    QrelsData {
        queries: vec![QrelsQuery {
            qid: "1".into(),
            records: vec![QrelsRecord {
                docno: "a".into(),
                rel: 1,
            }],
        }],
        comments: vec![],
        warnings: vec![],
    }
}

fn run() -> RunData {
    RunData {
        run_id: "r".into(),
        queries: vec![RunQuery {
            qid: "1".into(),
            records: vec![
                RunRecord {
                    docno: "a".into(),
                    sim: 2.0,
                },
                RunRecord {
                    docno: "a".into(),
                    sim: 1.0,
                },
            ],
        }],
        comments: vec![],
        warnings: vec![],
    }
}

#[test]
fn library_refuses_scored_run_duplicates() {
    let measures = resolve_measures(&["map".into(), "recall.2".into()]).unwrap();
    let result = evaluate_run(&qrels(), &run(), &measures, &EvalConfig::default(), None);
    assert_eq!(
        result.unwrap_err(),
        EvaluationError::Duplicate(ScoredDuplicate::Run {
            qid: "1".into(),
            docno: "a".into()
        })
    );
}

#[test]
fn library_refuses_scored_jg_run_duplicates() {
    let qrels = QrelsJGData {
        queries: vec![QrelsJGQuery {
            qid: "1".into(),
            groups: vec![QrelsJGGroup {
                jg: "g".into(),
                records: qrels().queries.remove(0).records,
            }],
        }],
        comments: vec![],
        warnings: vec![],
    };
    let measures = resolve_measures(&["map_avgjg".into()]).unwrap();
    let result = evaluate_run_jg(&qrels, &run(), &measures, &EvalConfig::default(), None);
    assert_eq!(
        result.unwrap_err(),
        EvaluationError::Duplicate(ScoredDuplicate::Run {
            qid: "1".into(),
            docno: "a".into()
        })
    );
}

#[test]
fn library_checks_only_retained_ranks_and_requested_rankings() {
    for (request, max_docs) in [("map", 1), ("num_q", usize::MAX), ("runid", usize::MAX)] {
        let measures = resolve_measures(&[request.into()]).unwrap();
        let config = EvalConfig {
            max_num_docs_per_topic: max_docs,
            ..Default::default()
        };
        assert!(evaluate_run(&qrels(), &run(), &measures, &config, None).is_ok());
    }
}

#[test]
fn library_checks_missing_qrels_topics_only_with_complete_averaging() {
    let mut qrels = qrels();
    qrels.queries.push(QrelsQuery {
        qid: "2".into(),
        records: vec![
            QrelsRecord {
                docno: "b".into(),
                rel: 1
            };
            2
        ],
    });
    let mut run = run();
    run.queries[0].records.truncate(1);
    let measures = resolve_measures(&["map".into()]).unwrap();
    assert!(evaluate_run(&qrels, &run, &measures, &EvalConfig::default(), None).is_ok());
    let config = EvalConfig {
        average_complete_flag: true,
        ..Default::default()
    };
    assert_eq!(
        evaluate_run(&qrels, &run, &measures, &config, None).unwrap_err(),
        EvaluationError::Duplicate(ScoredDuplicate::Qrels {
            qid: "2".into(),
            docno: "b".into()
        })
    );
}
