use std::path::PathBuf;
use std::process::Command;

use te_rust::metrics::registry::resolve_measures;

#[test]
fn both_rbp_measures_refuse_repeated_p() {
    for measure in ["rbp", "rbp_resid"] {
        let spec = format!("{measure}.p=0.2,p=0.8");
        assert!(
            resolve_measures(std::slice::from_ref(&spec)).is_err(),
            "{spec}"
        );
    }
}

#[test]
fn both_rbp_measures_require_integer_gain_names() {
    for measure in ["rbp", "rbp_resid"] {
        for name in ["x", "P", "1x", "1.0"] {
            let spec = format!("{measure}.{name}=1");
            assert!(
                resolve_measures(std::slice::from_ref(&spec)).is_err(),
                "{spec}"
            );
        }
    }
}

#[test]
fn both_rbp_measures_refuse_duplicate_levels() {
    for measure in ["rbp", "rbp_resid"] {
        for gains in ["1=2,01=3", "1=2,+1=3", "-1=2,-01=3"] {
            let spec = format!("{measure}.{gains}");
            assert!(
                resolve_measures(std::slice::from_ref(&spec)).is_err(),
                "{spec}"
            );
        }
    }
}

#[test]
fn unusable_rbp_parameters_exit_two() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rbp-gain-range");
    for spec in [
        "rbp.p=0.2,p=0.8",
        "rbp_resid.p=0.2,p=0.8",
        "rbp_resid.x=1",
        "rbp_resid.1=2,01=3",
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_te-rust"))
            .args(["-m", spec])
            .arg(fixtures.join("one.qrels"))
            .arg(fixtures.join("one.run"))
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{spec}: {out:?}");
        assert!(out.stdout.is_empty(), "{spec}: {out:?}");
    }
}

#[test]
fn residual_accepts_integer_gain_levels_without_using_them() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rbp-gain-range");
    let out = Command::new(env!("CARGO_BIN_EXE_te-rust"))
        .args(["-m", "rbp_resid.p=0.5,-1=10,01=2"])
        .arg(fixtures.join("one.qrels"))
        .arg(fixtures.join("unjudged.run"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.split_whitespace().last(), Some("0.7500"));
}
