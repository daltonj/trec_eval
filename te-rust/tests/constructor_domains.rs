use te_rust::metrics::{rbp::RbpMeasure, rbp_resid::RbpResidMeasure, set_f::SetFMeasure};

#[test]
fn rbp_constructors_require_finite_persistence_in_the_unit_interval() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 1.0, 2.0] {
        assert!(RbpMeasure::new(bad, "").is_err(), "rbp {bad}");
        assert!(
            std::panic::catch_unwind(|| RbpResidMeasure::new(bad, "")).is_err(),
            "rbp_resid {bad}"
        );
    }
    for good in [0.0, 0.9, 1.0 - f64::EPSILON] {
        RbpMeasure::new(good, "").unwrap();
        RbpResidMeasure::new(good, "");
    }
}

#[test]
fn set_f_constructor_requires_a_finite_nonnegative_weight() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1] {
        assert!(SetFMeasure::new(bad, "").is_err(), "set_F {bad}");
    }
    for good in [0.0, 1.0, 1e300] {
        SetFMeasure::new(good, "").unwrap();
    }
}
