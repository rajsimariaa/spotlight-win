pub mod app_index;
pub mod everything;
pub mod fuzzy;
pub mod evaluator;
pub mod quick_actions;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub path: String,
    pub category: SearchResultCategory,
    pub icon: Option<String>,
    pub score: f64,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchResultCategory {
    Application,
    File,
    Calculator,
    Action,
    Conversion,
    Timezone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub query_time_ms: u64,
    pub total_results: usize,
}
