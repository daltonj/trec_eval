use std::path::PathBuf;

use te_rust::eval::evaluate_run;
use te_rust::io::{parse_trec_qrels, parse_trec_run};
use te_rust::metrics::iprec_at_recall::IprecAtRecallMeasure;
use te_rust::metrics::map_cut::MapCutMeasure;
use te_rust::metrics::ndcg_cut::NdcgCutMeasure;
use te_rust::metrics::precision::PrecisionCutMeasure;
use te_rust::metrics::precision_avgjg::PrecisionAvgjgMeasure;
use te_rust::metrics::recall::RecallCutMeasure;
use te_rust::metrics::relative_p::RelativePMeasure;
use te_rust::metrics::rprec_mult::RprecMultMeasure;
use te_rust::metrics::rprec_mult_avgjg::RprecMultAvgjgMeasure;
use te_rust::metrics::success::SuccessCutMeasure;
use te_rust::metrics::unj::UnjMeasure;
use te_rust::metrics::{EvalConfig, Measure, MetricValue};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/constructor-cutoffs")
        .join(name)
}

/// Summary rows as (name, value to four decimals), in output order.
fn summary(name: &str, measure: Box<dyn Measure>) -> Vec<(String, String)> {
    let qrels = parse_trec_qrels(fixture(&format!("{name}.qrels"))).unwrap();
    let run = parse_trec_run(fixture(&format!("{name}.run"))).unwrap();
    let out = evaluate_run(&qrels, &run, &[measure], &EvalConfig::default(), None).unwrap();
    out.summary_results
        .into_iter()
        .map(|row| match row.value {
            MetricValue::Float(v) => (row.name, format!("{v:.4}")),
            other => panic!("{}: {other:?}", row.name),
        })
        .collect()
}

#[test]
fn map_cut_from_an_unsorted_list_matches_hand_calculation() {
    // Six relevant documents, all retrieved in the top six. map_cut_2 = (1 + 1) / 6 and
    // map_cut_5 = 5 / 6. With the list kept as given, map_cut_2 took the value at the end of
    // the ranking.
    let rows = summary(
        "six-relevant",
        Box::new(MapCutMeasure::new(vec![5, 2]).unwrap()),
    );
    let expected = [("map_cut_2", "0.3333"), ("map_cut_5", "0.8333")];
    assert_eq!(rows, expected.map(|(n, v)| (n.to_string(), v.to_string())));
}

#[test]
fn every_constructor_sorts_its_cutoffs() {
    let pairs: Vec<(Box<dyn Measure>, Box<dyn Measure>)> = vec![
        (
            Box::new(MapCutMeasure::new(vec![5, 2]).unwrap()),
            Box::new(MapCutMeasure::new(vec![2, 5]).unwrap()),
        ),
        (
            Box::new(NdcgCutMeasure::new(vec![5, 2]).unwrap()),
            Box::new(NdcgCutMeasure::new(vec![2, 5]).unwrap()),
        ),
        (
            Box::new(RelativePMeasure::new(vec![5, 2]).unwrap()),
            Box::new(RelativePMeasure::new(vec![2, 5]).unwrap()),
        ),
        (
            Box::new(UnjMeasure::new(vec![5, 2]).unwrap()),
            Box::new(UnjMeasure::new(vec![2, 5]).unwrap()),
        ),
        (
            Box::new(PrecisionCutMeasure::new(vec![5, 2]).unwrap()),
            Box::new(PrecisionCutMeasure::new(vec![2, 5]).unwrap()),
        ),
        (
            Box::new(RecallCutMeasure::new(vec![5, 2]).unwrap()),
            Box::new(RecallCutMeasure::new(vec![2, 5]).unwrap()),
        ),
        (
            Box::new(SuccessCutMeasure::new(vec![5, 2]).unwrap()),
            Box::new(SuccessCutMeasure::new(vec![2, 5]).unwrap()),
        ),
        (
            Box::new(IprecAtRecallMeasure::new(vec![0.8, 0.2]).unwrap()),
            Box::new(IprecAtRecallMeasure::new(vec![0.2, 0.8]).unwrap()),
        ),
        (
            Box::new(RprecMultMeasure::new(vec![2.0, 0.5]).unwrap()),
            Box::new(RprecMultMeasure::new(vec![0.5, 2.0]).unwrap()),
        ),
    ];
    for (given, sorted) in pairs {
        let name = given.name();
        assert_eq!(summary("mixed", given), summary("mixed", sorted), "{name}");
    }
    // The qrels_jg measures print their rows in the same ascending order.
    assert_eq!(
        PrecisionAvgjgMeasure::new(vec![5, 2])
            .unwrap()
            .sub_metrics(),
        ["P_avgjg_2", "P_avgjg_5"]
    );
    assert_eq!(
        RprecMultAvgjgMeasure::new(vec![2.0, 0.5])
            .unwrap()
            .sub_metrics(),
        ["Rprec_mult_avgjg_0.50", "Rprec_mult_avgjg_2.00"]
    );
}

#[test]
fn zero_rank_cutoff_is_refused() {
    let error = MapCutMeasure::new(vec![5, 0]).err().unwrap();
    assert_eq!(error.message, "rank cutoff must be at least 1");
}

#[test]
fn repeated_rank_cutoff_is_refused() {
    let error = PrecisionCutMeasure::new(vec![5, 2, 5]).err().unwrap();
    assert_eq!(error.message, "cutoff 5 is given more than once");
}

#[test]
fn repeated_float_cutoff_is_refused() {
    let error = RprecMultMeasure::new(vec![1.0, 0.5, 1.0]).err().unwrap();
    assert_eq!(error.message, "cutoff 1 is given more than once");
}

#[test]
fn float_constructors_refuse_non_finite_values_and_name_collisions() {
    for bad in [
        vec![f64::NAN],
        vec![f64::INFINITY],
        vec![f64::NEG_INFINITY],
        vec![0.101, 0.104],
    ] {
        assert!(
            IprecAtRecallMeasure::new(bad.clone()).is_err(),
            "iprec {bad:?}"
        );
        assert!(
            RprecMultMeasure::new(bad.clone()).is_err(),
            "Rprec_mult {bad:?}"
        );
        assert!(
            RprecMultAvgjgMeasure::new(bad.clone()).is_err(),
            "Rprec_mult_avgjg {bad:?}"
        );
    }
}

#[test]
fn recall_constructors_refuse_levels_outside_the_unit_interval() {
    use te_rust::metrics::avg_11pt::Avg11PtMeasure;
    for bad in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
        assert!(IprecAtRecallMeasure::new(vec![bad]).is_err(), "iprec {bad}");
        assert!(
            Avg11PtMeasure::new(vec![bad], "").is_err(),
            "11pt_avg {bad}"
        );
    }
    Avg11PtMeasure::new(vec![0.0, 0.0, 0.101, 0.104, 1.0], "").unwrap();
    IprecAtRecallMeasure::new(vec![0.0, 1.0]).unwrap();
}

#[test]
fn rprec_constructors_require_positive_multiples() {
    for bad in [-1.0, 0.0] {
        assert!(RprecMultMeasure::new(vec![bad]).is_err());
        assert!(RprecMultAvgjgMeasure::new(vec![bad]).is_err());
    }
    RprecMultMeasure::new(vec![f64::MIN_POSITIVE, 1e300]).unwrap();
}

#[test]
#[cfg(target_pointer_width = "64")]
fn rank_constructors_refuse_cutoffs_above_i64_max() {
    assert!(PrecisionCutMeasure::new(vec![i64::MAX as usize + 1]).is_err());
    PrecisionCutMeasure::new(vec![i64::MAX as usize]).unwrap();
}
