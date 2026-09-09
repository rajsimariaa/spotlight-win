use super::{SearchResult, SearchResultCategory};

pub fn evaluate_expression(input: &str) -> Option<SearchResult> {
    let trimmed = input.trim();

    // Try math evaluation
    if let Some(result) = evaluate_math(trimmed) {
        return Some(result);
    }

    // Try timezone conversion
    if let Some(result) = evaluate_timezone(trimmed) {
        return Some(result);
    }

    // Try unit conversion
    if let Some(result) = evaluate_unit_conversion(trimmed) {
        return Some(result);
    }

    None
}

fn evaluate_math(input: &str) -> Option<SearchResult> {
    // Check if input looks like a math expression
    if !input.chars().any(|c| "+-*/^%()".contains(c)) && !input.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }

    match evalexpr::eval(input) {
        Ok(value) => {
            let result_str = match value {
                evalexpr::Value::Int(i) => i.to_string(),
                evalexpr::Value::Float(f) => {
                    if f.fract() == 0.0 && f.abs() < i64::MAX as f64 {
                        (f as i64).to_string()
                    } else {
                        format!("{:.6}", f).trim_end_matches('0').trim_end_matches('.').to_string()
                    }
                }
                evalexpr::Value::Boolean(b) => b.to_string(),
                _ => return None,
            };

            let display = format!("{} = {}", input, result_str);

            Some(SearchResult {
                id: format!("calc_{}", input),
                name: result_str.clone(),
                path: result_str,
                category: SearchResultCategory::Calculator,
                icon: Some("calculator".to_string()),
                score: 100.0,
                metadata: Some(display),
            })
        }
        Err(_) => None,
    }
}

fn evaluate_timezone(input: &str) -> Option<SearchResult> {
    let lower = input.to_lowercase();

    // Pattern: "time in Tokyo" or "time London"
    let patterns = [
        "time in ",
        "time ",
        "what time is it in ",
        "current time ",
    ];

    for pattern in &patterns {
        if let Some(tz_name) = lower.strip_prefix(pattern) {
            let tz_name = tz_name.trim();
            if let Ok(tz) = tz_name.parse::<chrono_tz::Tz>() {
                let now = chrono::Utc::now();
                let local_time = now.with_timezone(&tz);
                let time_str = local_time.format("%I:%M:%S %p %Z").to_string();
                let date_str = local_time.format("%A, %B %d, %Y").to_string();
                let path_str = format!("{}, {}", time_str, date_str);
                let metadata_str = format!("Current time in {}: {}", tz_name, time_str);

                return Some(SearchResult {
                    id: format!("tz_{}", tz_name),
                    name: time_str,
                    path: path_str,
                    category: SearchResultCategory::Timezone,
                    icon: Some("clock".to_string()),
                    score: 80.0,
                    metadata: Some(metadata_str),
                });
            }
        }
    }

    None
}

fn evaluate_unit_conversion(input: &str) -> Option<SearchResult> {
    let lower = input.to_lowercase();

    // Temperature conversions (simple patterns first)
    if let Some(caps) = regex_lite::Regex::new(r"(\d+(?:\.\d+)?)\s*(?:c|celsius|°c)\s+(?:in|to|is)\s+(?:f|fahrenheit|°f)").ok()?.captures(&lower) {
        let value: f64 = caps.get(1)?.as_str().parse().ok()?;
        let result = value * 9.0 / 5.0 + 32.0;
        let display = format!("{}°C = {:.1}°F", value, result);
        return Some(SearchResult {
            id: format!("temp_{}_{}", input, result),
            name: display.clone(),
            path: display,
            category: SearchResultCategory::Conversion,
            icon: Some("thermometer".to_string()),
            score: 80.0,
            metadata: Some(format!("Temperature conversion")),
        });
    }

    if let Some(caps) = regex_lite::Regex::new(r"(\d+(?:\.\d+)?)\s*(?:f|fahrenheit|°f)\s+(?:in|to|is)\s+(?:c|celsius|°c)").ok()?.captures(&lower) {
        let value: f64 = caps.get(1)?.as_str().parse().ok()?;
        let result = (value - 32.0) * 5.0 / 9.0;
        let display = format!("{}°F = {:.1}°C", value, result);
        return Some(SearchResult {
            id: format!("temp_{}_{}", input, result),
            name: display.clone(),
            path: display,
            category: SearchResultCategory::Conversion,
            icon: Some("thermometer".to_string()),
            score: 80.0,
            metadata: Some(format!("Temperature conversion")),
        });
    }

    // Distance conversions
    if let Some(caps) = regex_lite::Regex::new(r"(\d+(?:\.\d+)?)\s*(km|kilometers?)\s+(?:in|to)\s+(mi|miles?|m|meters?|ft|feet|yds?|yards?)").ok()?.captures(&lower) {
        let value: f64 = caps.get(1)?.as_str().parse().ok()?;
        let from_unit = caps.get(2)?.as_str();
        let to_unit = caps.get(3)?.as_str();
        let result = convert_units(value, from_unit, to_unit)?;
        let display = format!("{} {} = {:.2} {}", value, from_unit, result, to_unit);
        return Some(SearchResult {
            id: format!("unit_{}_{}", input, result),
            name: display.clone(),
            path: display,
            category: SearchResultCategory::Conversion,
            icon: Some("ruler".to_string()),
            score: 80.0,
            metadata: Some(format!("Unit conversion")),
        });
    }

    // Weight conversions
    if let Some(caps) = regex_lite::Regex::new(r"(\d+(?:\.\d+)?)\s*(kg|kilograms?)\s+(?:in|to)\s+(lb|lbs?|pounds?|oz|ounces?)").ok()?.captures(&lower) {
        let value: f64 = caps.get(1)?.as_str().parse().ok()?;
        let from_unit = caps.get(2)?.as_str();
        let to_unit = caps.get(3)?.as_str();
        let result = convert_units(value, from_unit, to_unit)?;
        let display = format!("{} {} = {:.2} {}", value, from_unit, result, to_unit);
        return Some(SearchResult {
            id: format!("unit_{}_{}", input, result),
            name: display.clone(),
            path: display,
            category: SearchResultCategory::Conversion,
            icon: Some("ruler".to_string()),
            score: 80.0,
            metadata: Some(format!("Unit conversion")),
        });
    }

    None
}

fn convert_units(value: f64, from: &str, to: &str) -> Option<f64> {
    let conversion_table = [
        // Distance to meters
        ("km", 1000.0),
        ("kilometer", 1000.0),
        ("kilometers", 1000.0),
        ("mi", 1609.344),
        ("mile", 1609.344),
        ("miles", 1609.344),
        ("m", 1.0),
        ("meter", 1.0),
        ("meters", 1.0),
        ("ft", 0.3048),
        ("feet", 0.3048),
        ("yd", 0.9144),
        ("yard", 0.9144),
        ("yards", 0.9144),
        ("in", 0.0254),
        ("inch", 0.0254),
        ("inches", 0.0254),
        // Weight to kg
        ("kg", 1.0),
        ("kilogram", 1.0),
        ("kilograms", 1.0),
        ("lb", 0.453592),
        ("lbs", 0.453592),
        ("pound", 0.453592),
        ("pounds", 0.453592),
        ("oz", 0.0283495),
        ("ounce", 0.0283495),
        ("ounces", 0.0283495),
        ("g", 0.001),
        ("gram", 0.001),
        ("grams", 0.001),
    ];

    let from_lower = from.to_lowercase();
    let to_lower = to.to_lowercase();

    let from_factor = conversion_table.iter()
        .find(|(u, _)| *u == from_lower.as_str())
        .map(|(_, f)| f)?;

    let to_factor = conversion_table.iter()
        .find(|(u, _)| *u == to_lower.as_str())
        .map(|(_, f)| f)?;

    Some(value * from_factor / to_factor)
}
