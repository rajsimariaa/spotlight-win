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

    // Token-based: query "my rep" matches "my_report" or "MyReport"
    let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();
    if query_tokens.len() > 1 {
        let mut all_matched = true;
        let mut token_score = 0.0;
        for token in &query_tokens {
            if let Some(pos) = target_lower.find(token) {
                token_score += 20.0 + (1.0 - pos as f64 / target.len() as f64) * 10.0;
            } else {
                // Try fuzzy within token
                let mut found = false;
                for i in 0..target_lower.len() {
                    let remaining = &target_lower[i..];
                    if fuzzy_contains_token(remaining, token) {
                        token_score += 15.0;
                        found = true;
                        break;
                    }
                }
                if !found {
                    all_matched = false;
                    break;
                }
            }
        }
        if all_matched {
            return 50.0 + token_score;
        }
    }

    // Path-aware: match against path segments
    let path_segments: Vec<&str> = target_lower.split(&['\\', '/'][..]).collect();
    for segment in &path_segments {
        if segment.starts_with(&query_lower) {
            return 45.0;
        }
        if fuzzy_contains_subsequence(segment, &query_lower) {
            return 40.0;
        }
    }

    // Fuzzy character matching with word-boundary awareness
    let mut score = 0.0;
    let mut query_idx = 0;
    let mut last_match_idx = -1i32;
    let mut consecutive_bonus = 0.0;
    let mut match_count = 0;

    for (idx, target_char) in target_lower.chars().enumerate() {
        if query_idx >= query_lower.len() {
            break;
        }

        let query_char = query_lower.chars().nth(query_idx).unwrap_or('\0');

        if target_char == query_char {
            score += 10.0;
            match_count += 1;

            // Consecutive bonus
            if last_match_idx >= 0 && idx as i32 == last_match_idx + 1 {
                consecutive_bonus += 5.0;
            } else {
                consecutive_bonus = 0.0;
            }

            // Word boundary bonus (space, -, _, /, \, .)
            if idx == 0 || target_lower.chars().nth(idx - 1) == Some(' ')
                || target_lower.chars().nth(idx - 1) == Some('-')
                || target_lower.chars().nth(idx - 1) == Some('_')
                || target_lower.chars().nth(idx - 1) == Some('/')
                || target_lower.chars().nth(idx - 1) == Some('\\')
                || target_lower.chars().nth(idx - 1) == Some('.')
            {
                score += 20.0;
            }

            // camelCase bonus
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

    score += consecutive_bonus;

    // Match density bonus: shorter targets with all chars matched score higher
    let density = match_count as f64 / target.len() as f64;
    score += density * 10.0;

    // Bonus if all consecutive
    if consecutive_bonus > 0.0 {
        score += 5.0;
    }

    score
}

/// Check if target contains all characters of token in order
fn fuzzy_contains_subsequence(target: &str, token: &str) -> bool {
    let mut t_idx = 0;
    for ch in token.chars() {
        let found = target[t_idx..].find(ch);
        match found {
            Some(pos) => t_idx += pos + 1,
            None => return false,
        }
    }
    true
}

/// Check if remaining string starts with the fuzzy token
fn fuzzy_contains_token(remaining: &str, token: &str) -> bool {
    let mut r_iter = remaining.chars();
    for ch in token.chars() {
        match r_iter.find(|&c| c == ch) {
            Some(_) => {}
            None => return false,
        }
    }
    true
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

    #[test]
    fn test_token_match() {
        let score = calculate_fuzzy_score("my rep", "my_report.pdf");
        assert!(score > 50.0, "Token match should score > 50, got {}", score);
    }

    #[test]
    fn test_path_segments() {
        let score = calculate_fuzzy_score("desktop", "C:\\Users\\me\\Desktop\\file.txt");
        assert!(score > 40.0, "Path segment match should score > 40, got {}", score);
    }
}
