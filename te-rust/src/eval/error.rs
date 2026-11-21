use crate::io::parsers::REL_LEVELS;

/// The file that repeats a document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DuplicateSource {
    Qrels,
    Run,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationError {
    /// The relevance threshold is negative.
    NegativeRelevanceLevel { value: i64 },
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
    /// A run's score is NaN, which has no place in the ranking.
    NanScore { qid: String, docno: String },
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NegativeRelevanceLevel { value } => {
                write!(f, "Unusable relevance level '{}'", value)
            }
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
            Self::NanScore { qid, docno } => {
                write!(f, "score of document '{}' for query '{}' is NaN", docno, qid)
            }
        }
    }
}

impl std::error::Error for EvaluationError {}
