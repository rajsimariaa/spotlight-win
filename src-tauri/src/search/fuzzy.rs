pub fn calculate_fuzzy_score(query: &str, target: &str) -> f64 {
    if query.is_empty() {
        return 1.0;
    }

    let query_lower = query.to_lowercase();
    let target_lower = target.to_lowercase();

    // Exact match
    if target_lower == query_lower {
        return 100.0;
    }

    // Starts with query
    if target_lower.starts_with(&query_lower) {
        return 90.0 + (query.len() as f64 / target.len() as f64) * 10.0;
    }

    // Contains query as substring
    if let Some(pos) = target_lower.find(&query_lower) {
        return 70.0 + (1.0 - pos as f64 / target.len() as f64) * 20.0;
    }

    // Fuzzy character matching
    let mut score = 0.0;
    let mut query_idx = 0;
    let mut last_match_idx = -1i32;
    let mut consecutive_bonus = 0.0;

    for (idx, target_char) in target_lower.chars().enumerate() {
        if query_idx >= query_lower.len() {
            break;
        }

        let query_char = query_lower.chars().nth(query_idx).unwrap_or('\0');

        if target_char == query_char {
            // Base score for match
            score += 10.0;

            // Bonus for consecutive matches
            if last_match_idx >= 0 && idx as i32 == last_match_idx + 1 {
                consecutive_bonus += 5.0;
            } else {
                consecutive_bonus = 0.0;
            }

            // Bonus for word boundary match
            if idx == 0 || target_lower.chars().nth(idx - 1) == Some(' ') || target_lower.chars().nth(idx - 1) == Some('-') || target_lower.chars().nth(idx - 1) == Some('_') {
                score += 15.0;
            }

            // Bonus for camelCase match
            if idx > 0 && target.chars().nth(idx).map_or(false, |c| c.is_uppercase()) {
                score += 10.0;
            }

            last_match_idx = idx as i32;
            query_idx += 1;
        }
    }

    // All query characters must match
    if query_idx < query_lower.len() {
        return 0.0;
    }

    // Apply consecutive bonus
    score += consecutive_bonus;

    // Length penalty - shorter targets score higher
    let length_penalty = query.len() as f64 / target.len() as f64;
    score *= length_penalty;

    // Prefix bonus
    if target_lower.starts_with(query_lower.chars().next().unwrap_or('\0')) {
        score += 5.0;
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(calculate_fuzzy_score("chrome", "chrome") > 95.0);
    }

    #[test]
    fn test_starts_with() {
        assert!(calculate_fuzzy_score("chr", "chrome") > 85.0);
    }

    #[test]
    fn test_contains() {
        assert!(calculate_fuzzy_score("rom", "chrome") > 60.0);
    }

    #[test]
    fn test_fuzzy_match() {
        assert!(calculate_fuzzy_score("cme", "chrome") > 0.0);
    }

    #[test]
    fn test_no_match() {
        assert_eq!(calculate_fuzzy_score("xyz", "chrome"), 0.0);
    }
}
