use te_rust::metrics::{
    avg_11pt::Avg11PtMeasure, iprec_at_recall::IprecAtRecallMeasure,
    precision::PrecisionCutMeasure, rprec_mult::RprecMultMeasure,
    rprec_mult_avgjg::RprecMultAvgjgMeasure,
};

#[test]
fn constructors_refuse_empty_cutoff_vectors() {
    assert!(Avg11PtMeasure::new(vec![], "").is_err());
    assert!(PrecisionCutMeasure::new(vec![]).is_err());
    assert!(IprecAtRecallMeasure::new(vec![]).is_err());
    assert!(RprecMultMeasure::new(vec![]).is_err());
    assert!(RprecMultAvgjgMeasure::new(vec![]).is_err());
}
