/// A document that appears twice in a topic that would be scored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoredDuplicate {
    Qrels { qid: String, docno: String },
    Run { qid: String, docno: String },
}

impl std::fmt::Display for ScoredDuplicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (ScoredDuplicate::Qrels { qid, docno } | ScoredDuplicate::Run { qid, docno }) = self;
        write!(f, "duplicate document '{}' for query '{}'", docno, qid)
    }
}

impl std::error::Error for ScoredDuplicate {}
