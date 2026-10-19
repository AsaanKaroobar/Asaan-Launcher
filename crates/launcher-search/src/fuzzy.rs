// Ported from Floe Search/Fuzzy.swift and Ranking.swift.
// Copyright (Floe) © 2026 René Jiménez. Licensed under GNU AGPLv3.
// Changes: Rust implementation, normalized extended grapheme clusters, same scoring/offset contract.
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Match {
    pub score: i32,
    pub matched: Vec<usize>,
}

trait Unit: PartialEq {
    fn word(&self) -> bool;
    fn upper(&self) -> bool;
    fn lower(&self) -> bool;
}
impl Unit for u8 {
    fn word(&self) -> bool {
        self.is_ascii_alphanumeric()
    }
    fn upper(&self) -> bool {
        self.is_ascii_uppercase()
    }
    fn lower(&self) -> bool {
        self.is_ascii_lowercase()
    }
}
impl Unit for String {
    fn word(&self) -> bool {
        self.chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c.is_numeric())
    }
    fn upper(&self) -> bool {
        self != &lowercase(self) && self == &self.to_uppercase().nfc().collect::<String>()
    }
    fn lower(&self) -> bool {
        self != &self.to_uppercase() && self == &lowercase(self).nfc().collect::<String>()
    }
}

/// Swift's locale-independent casing maps scalars without Rust's final-sigma context rule.
pub fn lowercase(text: &str) -> String {
    text.chars().flat_map(char::to_lowercase).collect()
}

pub fn canonical_prefix(text: &str, prefix: &str) -> bool {
    characters(text).starts_with(&characters(prefix))
}

fn characters(text: &str) -> Vec<String> {
    text.graphemes(true).map(|g| g.nfc().collect()).collect()
}

pub fn match_text(query: &str, candidate: &str) -> Option<Match> {
    evaluate(query, candidate, true, true)
}
pub fn score(query: &str, candidate: &str) -> Option<i32> {
    evaluate(query, candidate, false, true).map(|m| m.score)
}
pub fn tier_score(query: &str, candidate: &str) -> Option<i32> {
    evaluate(query, candidate, false, false).map(|m| m.score)
}

fn evaluate(query: &str, candidate: &str, positions: bool, scattered: bool) -> Option<Match> {
    let plain = |s: &str| s.bytes().all(|b| b < 0x80 && b != b'\r');
    if plain(query) && plain(candidate) {
        let needle: Vec<_> = query.bytes().map(|b| b.to_ascii_lowercase()).collect();
        let text: Vec<_> = candidate.bytes().map(|b| b.to_ascii_lowercase()).collect();
        evaluate_units(&needle, &text, candidate.as_bytes(), positions, scattered)
    } else {
        evaluate_units(
            &characters(&lowercase(query)),
            &characters(&lowercase(candidate)),
            &characters(candidate),
            positions,
            scattered,
        )
    }
}

fn evaluate_units<U: Unit>(
    needle: &[U],
    text: &[U],
    original: &[U],
    positions: bool,
    allow_scattered: bool,
) -> Option<Match> {
    if needle.len() > text.len() {
        return None;
    }
    let mut found = 0;
    for unit in text {
        if found < needle.len() && *unit == needle[found] {
            found += 1;
        }
    }
    if found != needle.len() {
        return None;
    }
    let run = |start: usize| {
        if positions {
            (start..start + needle.len()).collect()
        } else {
            Vec::new()
        }
    };
    if text.starts_with(needle) {
        return Some(Match {
            score: 100 - (text.len() - needle.len()).min(20) as i32,
            matched: run(0),
        });
    }
    let starts: Vec<_> = (0..text.len())
        .filter(|&i| text[i].word() && (i == 0 || !text[i - 1].word()))
        .collect();
    if needle.iter().all(Unit::word)
        && let Some(&start) = starts.iter().find(|&&i| text[i..].starts_with(needle))
    {
        return Some(Match {
            score: 75,
            matched: run(start),
        });
    }
    if starts.len() >= needle.len() && starts.iter().zip(needle).all(|(&i, unit)| text[i] == *unit)
    {
        return Some(Match {
            score: 70,
            matched: if positions {
                starts[..needle.len()].to_vec()
            } else {
                vec![]
            },
        });
    }
    if let Some(start) = (0..=text.len() - needle.len()).find(|&i| text[i..].starts_with(needle)) {
        return Some(Match {
            score: 55,
            matched: run(start),
        });
    }
    if !allow_scattered {
        return None;
    }
    let humps = if original.len() == text.len() {
        original
    } else {
        text
    };
    let boundaries: Vec<_> = (0..text.len())
        .map(|i| {
            if i == 0 {
                text[i].word()
            } else {
                (text[i].word() && !text[i - 1].word())
                    || (humps[i].upper() && humps[i - 1].lower())
            }
        })
        .collect();
    Some(scattered_match(needle, text, &boundaries, positions))
}

fn scattered_match<U: Unit>(
    needle: &[U],
    text: &[U],
    boundaries: &[bool],
    positions: bool,
) -> Match {
    const UNREACHABLE: i64 = i64::MIN / 2;
    let count = needle.len();
    let length = text.len();
    let mut previous: Vec<i64> = (0..length)
        .map(|column| {
            if text[column] != needle[0] {
                UNREACHABLE
            } else {
                (if boundaries[column] { 40 } else { 0 })
                    + count as i64 * 4usize.saturating_sub(column) as i64
            }
        })
        .collect();
    let mut current = previous.clone();
    let mut origins = if positions {
        vec![0; count * length]
    } else {
        vec![]
    };
    for row in 1..count {
        let mut gap_best = UNREACHABLE;
        let mut gap_origin = 0;
        for column in 0..length {
            if column >= 2 && previous[column - 2] > gap_best {
                gap_best = previous[column - 2];
                gap_origin = column - 2;
            }
            if text[column] != needle[row] {
                current[column] = UNREACHABLE;
                continue;
            }
            let bonus: i64 = if boundaries[column] { 10 } else { 0 };
            let adjacent = if column >= 1 {
                previous[column - 1] + bonus.max(8) * 4
            } else {
                UNREACHABLE
            };
            let apart = gap_best + bonus * 4;
            current[column] = if adjacent.max(apart) < 0 {
                UNREACHABLE
            } else {
                adjacent.max(apart)
            };
            if positions {
                origins[row * length + column] = if apart > adjacent {
                    gap_origin
                } else {
                    column.saturating_sub(1)
                };
            }
        }
        std::mem::swap(&mut previous, &mut current);
    }
    let mut end = 0;
    for column in 1..length {
        if previous[column] > previous[end] {
            end = column;
        }
    }
    let score = (10 + previous[end] / count as i64).min(54) as i32;
    let mut matched = vec![];
    if positions {
        matched = vec![end; count];
        for row in (1..count).rev() {
            end = origins[row * length + end];
            matched[row - 1] = end;
        }
    }
    Match { score, matched }
}
