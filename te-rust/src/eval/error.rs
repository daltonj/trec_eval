use crate::io::parsers::REL_LEVELS;

/// The file that repeats a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateSource {
    Qrels,
    Run,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationError {
    /// A document appears twice in a topic that would be scored.
    DuplicateDocument {
        source: DuplicateSource,
        qid: String,
        docno: String,
    },
    /// A judgment's relevance level is outside `REL_LEVELS`.
    RelevanceOutOfBounds {
        qid: String,
        docno: String,
        value: i64,
    },
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateDocument { qid, docno, .. } => {
                write!(f, "duplicate document '{}' for query '{}'", docno, qid)
            }
            Self::RelevanceOutOfBounds { qid, docno, value } => write!(
                f,
                "relevance level {} for document '{}' in query '{}' is outside {} to {}",
                value,
                docno,
                qid,
                REL_LEVELS.start(),
                REL_LEVELS.end(),
            ),
        }
    }
}

impl std::error::Error for EvaluationError {}
