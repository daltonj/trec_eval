use te_rust::metrics::{
    avg_11pt::Avg11PtMeasure, iprec_at_recall::IprecAtRecallMeasure,
    precision::PrecisionCutMeasure, rprec_mult::RprecMultMeasure,
    rprec_mult_avgjg::RprecMultAvgjgMeasure,
};

#[test]
fn constructors_refuse_empty_cutoff_vectors() {
    assert!(std::panic::catch_unwind(|| Avg11PtMeasure::new(vec![], "")).is_err());
    assert!(std::panic::catch_unwind(|| PrecisionCutMeasure::new(vec![])).is_err());
    assert!(std::panic::catch_unwind(|| IprecAtRecallMeasure::new(vec![])).is_err());
    assert!(std::panic::catch_unwind(|| RprecMultMeasure::new(vec![])).is_err());
    assert!(std::panic::catch_unwind(|| RprecMultAvgjgMeasure::new(vec![])).is_err());
}
