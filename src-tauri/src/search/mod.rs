pub mod app_index;
pub mod fuzzy;
pub mod evaluator;
pub mod quick_actions;

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

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
    WebSearch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub query_time_ms: u64,
    pub total_results: usize,
}

static CACHED_APPS: OnceLock<Vec<app_index::AppEntry>> = OnceLock::new();

pub fn init_cache() {
    let apps = app_index::build_app_cache();
    let _ = CACHED_APPS.set(apps);
}

pub fn search(query: &str) -> SearchResponse {
    let start = std::time::Instant::now();
    let mut results: Vec<SearchResult> = Vec::new();

    if query.trim().is_empty() {
        return SearchResponse { results, query_time_ms: 0, total_results: 0 };
    }

    if let Some(apps) = CACHED_APPS.get() {
        results.extend(app_index::search_applications(query, apps));
    }

    if let Some(r) = evaluator::evaluate_expression(query) {
        results.push(r);
    }

    results.extend(quick_actions::search_actions(query));

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // Always add web search option at the end
    let q = query.trim().to_string();
    results.push(SearchResult {
        id: format!("web:{}", q),
        name: format!("Search \"{}\" on the web", q),
        path: format!("https://www.google.com/search?q={}", urlencoding::encode(&q)),
        category: SearchResultCategory::WebSearch,
        icon: None,
        score: -1.0,
        metadata: Some(format!("web:{}", q)),
    });

    let ms = start.elapsed().as_micros() as u64;
    let total = results.len();
    SearchResponse { results, query_time_ms: ms, total_results: total }
}
