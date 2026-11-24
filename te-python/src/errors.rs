use pyo3::exceptions::{PyUserWarning, PyValueError};
use pyo3::prelude::*;
use te_rust::eval::{DuplicateSource, EvaluationError};
use te_rust::io::parsers::repeated_document_lines;

use crate::conversions::InputFile;

pyo3::create_exception!(
    trec_eval._trec_eval,
    NoSharedTopicError,
    PyValueError,
    "Raised by `evaluate` and `evaluate_arrays` for a run that shares no topic with the qrels."
);

/// The Python exception for an evaluation error: `NoSharedTopicError` for a run that shares no
/// topic with the qrels, otherwise `ValueError`, naming the input, qrels or run, that repeats a
/// document.
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

/// As [`evaluation_error`], for qrels and a run that may have been read from the files given: a
/// document repeated in a file is reported with the file and, when the file can be read again,
/// the two lines, as the CLI reports it.
pub(crate) fn evaluation_error_in(
    error: EvaluationError,
    qrels_file: Option<&InputFile>,
    run_file: Option<&InputFile>,
) -> PyErr {
    if let EvaluationError::DuplicateDocument { source, qid, docno } = &error {
        let (kind, file) = match source {
            DuplicateSource::Qrels => ("qrels", qrels_file),
            DuplicateSource::Run => ("run", run_file),
        };
        if let Some(file) = file {
            return PyValueError::new_err(match repeated_document_lines(&file.path, qid, docno) {
                Some((first, second)) => format!(
                    "Error in {} file '{}': Line {}: {}, first at line {}",
                    kind, file.name, second, error, first
                ),
                None => format!("Error in {} file '{}': {}", kind, file.name, error),
            });
        }
    }
    evaluation_error(error)
}

/// The most warnings that one call of `warn_all` issues: a file with a problem on every line
/// would otherwise issue a warning per line, each kept in the caller's warning registry.
const MAX_WARNINGS: usize = 10;

/// Issues each warning as a Python `UserWarning`, up to `MAX_WARNINGS`, then one that counts the
/// rest.
pub(crate) fn warn_all(
    py: Python<'_>,
    warnings: impl IntoIterator<Item = impl AsRef<str>>,
) -> PyResult<()> {
    let category = py.get_type_bound::<PyUserWarning>();
    let mut warnings = warnings.into_iter();
    for warning in warnings.by_ref().take(MAX_WARNINGS) {
        PyErr::warn_bound(py, category.as_any(), warning.as_ref(), 1)?;
    }
    let rest = warnings.count();
    if rest > 0 {
        let summary = format!("{} more warnings like these were not issued", rest);
        PyErr::warn_bound(py, category.as_any(), &summary, 1)?;
    }
    Ok(())
}
