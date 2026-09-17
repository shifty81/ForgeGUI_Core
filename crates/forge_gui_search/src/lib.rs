//! Generic local/provider search contracts.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchRequest {
    pub query: String,
    pub filters: Vec<(String, String)>,
    pub limit: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub category: String,
    pub score: f32,
    pub metadata: Vec<(String, String)>,
}
pub trait SearchProvider {
    fn search(&self, request: &SearchRequest) -> Result<Vec<SearchHit>, String>;
}
pub fn rank_text(query: &str, title: &str, subtitle: &str) -> f32 {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return 0.5;
    }
    let t = title.to_lowercase();
    let s = subtitle.to_lowercase();
    if t == q {
        1.0
    } else if t.starts_with(&q) {
        0.9
    } else if t.contains(&q) {
        0.75
    } else if s.contains(&q) {
        0.5
    } else {
        0.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prefix_ranks_above_subtitle() {
        assert!(rank_text("for", "ForgeGUI", "") > rank_text("for", "x", "before"));
    }
}
