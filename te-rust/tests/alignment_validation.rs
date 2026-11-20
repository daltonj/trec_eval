use te_rust::eval::{align_query, align_query_jg};
use te_rust::io::{QrelsJGGroup, QrelsJGQuery, QrelsQuery, QrelsRecord};

fn qrels(rel: i64) -> QrelsQuery {
    QrelsQuery {
        qid: "1".into(),
        records: vec![QrelsRecord {
            docno: "a".into(),
            rel,
        }],
    }
}

#[test]
fn alignment_refuses_grades_outside_the_documented_range() {
    for grade in [i64::MAX, i64::MIN, 128, -128] {
        assert!(
            align_query(None, &qrels(grade), "r", 1, usize::MAX, false).is_err(),
            "{grade}"
        );
    }
}

#[test]
fn group_alignment_refuses_grades_outside_the_documented_range() {
    let qrels = QrelsJGQuery {
        qid: "1".into(),
        groups: vec![QrelsJGGroup {
            jg: "g".into(),
            records: qrels(i64::MAX).records,
        }],
    };
    assert!(align_query_jg(None, &qrels, "r", 1, usize::MAX, false).is_err());
}

#[test]
fn alignment_accepts_the_documented_grade_boundaries() {
    for grade in [-127, 0, 127] {
        let state = align_query(None, &qrels(grade), "r", 1, usize::MAX, false).unwrap();
        assert_eq!(state.num_rel, usize::from(grade >= 1));
        if grade >= 0 {
            assert_eq!(state.rel_levels[grade as usize], 1);
        }
    }
}
