use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontId, TextFormat};

pub fn build_highlight_job(text: &str, query: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let default_format = TextFormat {
        font_id: FontId::monospace(13.0),
        color: Color32::LIGHT_GRAY,
        ..Default::default()
    };
    let highlight_format = TextFormat {
        font_id: FontId::monospace(13.0),
        color: Color32::BLACK,
        background: Color32::from_rgb(244, 208, 63),
        ..Default::default()
    };

    if query.is_empty() {
        job.append(text, 0.0, default_format);
        return job;
    }

    let mut ranges = Vec::new();
    for token in query.split_whitespace().filter(|part| !part.is_empty()) {
        let token_ranges = find_case_insensitive_ranges(text, token);
        if token_ranges.is_empty() {
            ranges.extend(find_fuzzy_word_ranges(text, token));
        } else {
            ranges.extend(token_ranges);
        }
    }

    ranges.sort_unstable_by_key(|range| range.0);
    ranges.dedup();
    if ranges.is_empty() {
        job.append(text, 0.0, default_format);
        return job;
    }

    let merged = merge_ranges(&ranges);
    let mut cursor = 0usize;
    for (start, end) in merged {
        if start > cursor {
            job.append(&text[cursor..start], 0.0, default_format.clone());
        }
        if end > cursor {
            job.append(&text[start..end], 0.0, highlight_format.clone());
            cursor = end;
        }
    }

    if cursor < text.len() {
        job.append(&text[cursor..], 0.0, default_format);
    }

    job
}

pub fn find_all_match_positions(text: &str, query: &str) -> Vec<usize> {
    let query = query.trim();
    if query.is_empty() {
        return Vec::new();
    }

    let mut positions = Vec::new();
    for token in query.split_whitespace().filter(|token| !token.is_empty()) {
        let token_ranges = find_case_insensitive_ranges(text, token);
        if token_ranges.is_empty() {
            positions.extend(
                find_fuzzy_word_ranges(text, token)
                    .into_iter()
                    .map(|(start, _)| start),
            );
        } else {
            positions.extend(token_ranges.into_iter().map(|(start, _)| start));
        }
    }

    positions.sort_unstable();
    positions.dedup();
    positions
}

fn find_case_insensitive_ranges(text: &str, token: &str) -> Vec<(usize, usize)> {
    let token = token.trim();
    if token.is_empty() {
        return Vec::new();
    }

    let text_lower = text.to_ascii_lowercase();
    let token_lower = token.to_ascii_lowercase();

    let mut ranges = Vec::new();
    let mut cursor = 0usize;
    while let Some(offset) = text_lower[cursor..].find(&token_lower) {
        let start = cursor + offset;
        let end = start + token_lower.len();
        ranges.push((start, end));
        cursor = end;
    }

    ranges
}

fn find_fuzzy_word_ranges(text: &str, token: &str) -> Vec<(usize, usize)> {
    let token_lower = token.to_ascii_lowercase();
    if token_lower.len() < 3 {
        return Vec::new();
    }

    word_ranges(text)
        .into_iter()
        .filter(|(start, end)| {
            let word = text[*start..*end].to_ascii_lowercase();
            fuzzy_token_matches(&word, &token_lower)
        })
        .collect()
}

fn word_ranges(text: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut current_start = None;
    for (idx, ch) in text.char_indices() {
        if ch.is_alphanumeric() {
            if current_start.is_none() {
                current_start = Some(idx);
            }
            continue;
        }

        if let Some(start) = current_start.take() {
            ranges.push((start, idx));
        }
    }

    if let Some(start) = current_start {
        ranges.push((start, text.len()));
    }

    ranges
}

fn fuzzy_token_matches(word: &str, token: &str) -> bool {
    if word.contains(token) || is_subsequence(token.as_bytes(), word.as_bytes()) {
        return true;
    }

    let max_distance = if token.len() <= 4 { 1 } else { 2 };
    levenshtein_distance_at_most(word.as_bytes(), token.as_bytes(), max_distance)
        .is_some_and(|distance| distance <= max_distance)
}

fn is_subsequence(needle: &[u8], haystack: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }

    let mut needle_idx = 0usize;
    for byte in haystack {
        if *byte == needle[needle_idx] {
            needle_idx += 1;
            if needle_idx == needle.len() {
                return true;
            }
        }
    }

    false
}

fn levenshtein_distance_at_most(a: &[u8], b: &[u8], max_distance: usize) -> Option<usize> {
    let len_diff = a.len().abs_diff(b.len());
    if len_diff > max_distance {
        return None;
    }

    let mut prev_row: Vec<usize> = (0..=b.len()).collect();
    let mut current_row = vec![0usize; b.len() + 1];

    for (i, a_char) in a.iter().enumerate() {
        current_row[0] = i + 1;
        let mut row_min = current_row[0];

        for (j, b_char) in b.iter().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            let delete_cost = prev_row[j + 1] + 1;
            let insert_cost = current_row[j] + 1;
            let replace_cost = prev_row[j] + cost;

            let value = delete_cost.min(insert_cost).min(replace_cost);
            current_row[j + 1] = value;
            row_min = row_min.min(value);
        }

        if row_min > max_distance {
            return None;
        }

        prev_row.clone_from_slice(&current_row);
    }

    Some(prev_row[b.len()])
}

fn merge_ranges(ranges: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let mut merged = Vec::new();
    for &(start, end) in ranges {
        if let Some((_, prev_end)) = merged.last_mut() {
            if start <= *prev_end {
                *prev_end = (*prev_end).max(end);
                continue;
            }
        }
        merged.push((start, end));
    }
    merged
}

#[cfg(test)]
mod tests {
    use eframe::egui::Color32;

    use super::{build_highlight_job, find_all_match_positions};

    fn highlighted_fragments(text: &str, query: &str) -> Vec<String> {
        let highlighted_bg = Color32::from_rgb(244, 208, 63);
        let job = build_highlight_job(text, query);
        job.sections
            .iter()
            .filter(|section| section.format.background == highlighted_bg)
            .map(|section| job.text[section.byte_range.clone()].to_string())
            .collect()
    }

    #[test]
    fn highlights_case_insensitive_exact_matches() {
        let parts = highlighted_fragments("Rust fuzzy Search", "search");
        assert_eq!(parts, vec!["Search"]);
    }

    #[test]
    fn highlights_fuzzy_word_match_when_exact_token_is_missing() {
        let parts = highlighted_fragments("A fuzzy engine", "fzzy");
        assert_eq!(parts, vec!["fuzzy"]);
    }

    #[test]
    fn find_all_match_positions_finds_case_insensitive_matches() {
        let text = "line one\nTarget line\ntarget again";
        let positions = find_all_match_positions(text, "target");
        assert_eq!(positions, vec![9, 21]);
    }

    #[test]
    fn find_all_match_positions_uses_fuzzy_fallback() {
        let text = "alpha beta fuzzy gamma";
        let positions = find_all_match_positions(text, "fzzy");
        assert_eq!(positions, vec![11]);
    }

    #[test]
    fn find_all_match_positions_returns_empty_for_empty_query() {
        let positions = find_all_match_positions("text", "   ");
        assert!(positions.is_empty());
    }

    #[test]
    fn find_all_match_positions_returns_empty_when_no_match() {
        let positions = find_all_match_positions("line one\nline two", "nomatch");
        assert!(positions.is_empty());
    }
}
