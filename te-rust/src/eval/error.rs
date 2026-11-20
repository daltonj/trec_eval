use crate::eval::ScoredDuplicate;
use crate::io::parsers::REL_LEVELS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationError {
    Duplicate(ScoredDuplicate),
    RelevanceOutOfBounds {
        qid: String,
        docno: String,
        value: i64,
    },
}

impl From<ScoredDuplicate> for EvaluationError {
    fn from(duplicate: ScoredDuplicate) -> Self {
        Self::Duplicate(duplicate)
    }
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Duplicate(duplicate) => duplicate.fmt(f),
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
