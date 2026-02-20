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
        for (offset, _) in text.match_indices(token) {
            ranges.push((offset, offset + token.len()));
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
