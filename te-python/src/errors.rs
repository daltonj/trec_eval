use pyo3::exceptions::{PyUserWarning, PyValueError};
use pyo3::prelude::*;
use te_rust::eval::{DuplicateSource, EvaluationError};

pyo3::create_exception!(
    trec_eval._trec_eval,
    NoSharedTopicError,
    PyValueError,
    "Raised by `evaluate` and `evaluate_arrays` for a run that shares no topic with the qrels."
);

/// The Python exception for an evaluation error: `NoSharedTopicError` for a run that shares no
/// topic with the qrels, otherwise `ValueError`, naming the file that repeats a document.
pub(crate) fn evaluation_error(error: EvaluationError) -> PyErr {
    match &error {
        EvaluationError::NoSharedTopic => NoSharedTopicError::new_err(format!(
            "{}; complete_set_average=True evaluates each qrels topic missing from the run as an empty ranking",
            error
        )),
        EvaluationError::DuplicateDocument { source, .. } => {
            let kind = match source {
                DuplicateSource::Qrels => "qrels",
                DuplicateSource::Run => "run",
            };
            PyValueError::new_err(format!("{}: {}", kind, error))
        }
        _ => PyValueError::new_err(error.to_string()),
    }
}

/// Issues each warning as a Python `UserWarning`.
pub(crate) fn warn_all(
    py: Python<'_>,
    warnings: impl IntoIterator<Item = impl AsRef<str>>,
) -> PyResult<()> {
    let category = py.get_type_bound::<PyUserWarning>();
    for warning in warnings {
        PyErr::warn_bound(py, category.as_any(), warning.as_ref(), 1)?;
    }
    Ok(())
}
