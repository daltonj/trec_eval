use te_rust::metrics::utility::UtilityMeasure;

#[test]
fn utility_constructor_refuses_non_finite_coefficients() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(UtilityMeasure::new(vec![bad, 0.0, 0.0, 0.0], "").is_err());
    }
}

#[test]
fn utility_constructor_requires_four_coefficients() {
    for coeffs in [vec![], vec![1.0], vec![1.0; 5]] {
        assert!(UtilityMeasure::new(coeffs, "").is_err());
    }
    UtilityMeasure::new(vec![1e300, -1e300, 0.0, 1.0], "").unwrap();
}
