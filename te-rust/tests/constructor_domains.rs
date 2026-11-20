use te_rust::metrics::{rbp::RbpMeasure, rbp_resid::RbpResidMeasure, set_f::SetFMeasure};

#[test]
fn rbp_constructors_require_finite_persistence_in_the_unit_interval() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 1.0, 2.0] {
        let params = format!("p={bad}");
        assert!(RbpMeasure::new(&params).is_err(), "rbp {params}");
        assert!(RbpResidMeasure::new(&params).is_err(), "rbp_resid {params}");
    }
    for good in [0.0, 0.9, 1.0 - f64::EPSILON] {
        let params = format!("p={good}");
        RbpMeasure::new(&params).unwrap();
        RbpResidMeasure::new(&params).unwrap();
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
