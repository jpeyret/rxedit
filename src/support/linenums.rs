use crate::base::{CheckCondition, LineStatus};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    pub start: usize,
    pub end: usize,
}

impl Range {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, value: usize) -> bool {
        self.start <= value && value <= self.end
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[derive(Debug, Clone)]
pub struct CheckLineRange {
    /// New range-based representation for the DSL.
    pub where_linenum: String,
    pub ranges: Vec<Range>,
    pub hashset: HashSet<usize>,
}

impl CheckLineRange {
    pub fn new(where_linenum: String) -> Self {
        Self::with_total_lines(where_linenum, 0)
    }

    pub fn with_total_lines(where_linenum: String, total_lines: usize) -> Self {
        let mut ranges = Vec::new();
        let mut hashset = HashSet::new();

        if where_linenum.trim().is_empty() {
            return Self {
                where_linenum,
                ranges,
                hashset,
            };
        }

        for raw_segment in where_linenum.split(',') {
            let segment = raw_segment.trim();
            if segment.is_empty() {
                continue;
            }

            match parse_segment(segment, total_lines) {
                ParsedSegment::Range(range) => ranges.push(range),
                ParsedSegment::Line(value) => {
                    hashset.insert(value);
                }
                ParsedSegment::None => {}
            }
        }

        ranges.sort_by_key(|range| range.start);
        ranges.dedup_by(|current, next| current.start == next.start && current.end == next.end);

        let mut filtered_hashset = HashSet::new();
        for value in hashset {
            let covered = ranges.iter().any(|range| range.contains(value));
            if !covered {
                filtered_hashset.insert(value);
            }
        }

        Self {
            where_linenum,
            ranges,
            hashset: filtered_hashset,
        }
    }
}

impl Default for CheckLineRange {
    fn default() -> Self {
        Self {
            where_linenum: String::new(),
            ranges: Vec::new(),
            hashset: HashSet::new(),
        }
    }
}

impl CheckCondition for CheckLineRange {
    fn check(&self, line: &LineStatus) -> bool {
        self.ranges.iter().any(|range| range.contains(line.line_number))
            || self.hashset.contains(&line.line_number)
    }

    fn check_with_total_lines(&self, line: &LineStatus, total_lines: usize) -> bool {
        if self.where_linenum.is_empty() || total_lines == 0 {
            return self.check(line);
        }
        let resolved = Self::with_total_lines(self.where_linenum.clone(), total_lines);
        resolved.check(line)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

enum ParsedSegment {
    Range(Range),
    Line(usize),
    None,
}

fn parse_segment(segment: &str, total_lines: usize) -> ParsedSegment {
    let segment = segment.trim();

    if let Some(range) = parse_range_segment(segment, total_lines) {
        return ParsedSegment::Range(range);
    }

    if let Ok(value) = segment.parse::<usize>() {
        return ParsedSegment::Line(value);
    }

    if segment.starts_with('-') && segment.len() > 1 {
        if let Ok(offset) = segment[1..].parse::<usize>() {
            if total_lines > 0 {
                let value = total_lines.saturating_sub(offset).saturating_add(1);
                if value > 0 && value <= total_lines {
                    return ParsedSegment::Line(value);
                }
            }
        }
    }

    ParsedSegment::None
}

fn parse_range_segment(segment: &str, total_lines: usize) -> Option<Range> {
    if segment.contains("..") {
        let mut parts = segment.split("..");
        let left = parts.next().unwrap_or("");
        let right = parts.next().unwrap_or("");
        if parts.next().is_some() {
            return None;
        }

        let resolve = |raw: &str, is_right_bound: bool| -> Option<usize> {
            if raw.is_empty() {
                if is_right_bound {
                    return Some(total_lines.max(1));
                }
                return Some(1);
            }
            if let Ok(value) = raw.parse::<usize>() {
                return Some(value);
            }
            if raw.starts_with('-') && raw.len() > 1 {
                let offset = raw[1..].parse::<usize>().ok()?;
                if total_lines == 0 {
                    return None;
                }
                let value = total_lines.saturating_sub(offset).saturating_add(1);
                return Some(value.max(1));
            }
            None
        };

        let left_is_present = left != "";
        let right_is_present = right != "";

        let start = if left_is_present { resolve(left, false)? } else { 1 };
        let end = if right_is_present {
            resolve(right, true)?
        } else if total_lines == 0 {
            usize::MAX
        } else {
            total_lines
        };

        if start <= end {
            return Some(Range::new(start, end));
        }
        return None;
    }

    if segment.starts_with("h") && segment.len() > 1 {
        let count = segment[1..].parse::<usize>().ok()?;
        return Some(Range::new(1, count));
    }
    if segment == "h" {
        return Some(Range::new(1, 10));
    }
    if segment.starts_with("t") && segment.len() > 1 {
        let count = segment[1..].parse::<usize>().ok()?;
        if total_lines == 0 {
            return None;
        }
        let start = total_lines.saturating_sub(count).saturating_add(1).max(1);
        return Some(Range::new(start, total_lines));
    }
    if segment == "t" {
        if total_lines == 0 {
            return None;
        }
        let start = total_lines.saturating_sub(10).saturating_add(1).max(1);
        return Some(Range::new(start, total_lines));
    }

    if segment.contains('A') || segment.contains('B') || segment.contains('C') {
        let mut base = None;
        let mut offset = None;
        for marker in ['A', 'B', 'C'] {
            if let Some(index) = segment.find(marker) {
                let value = &segment[index + 1..];
                if let Ok(parsed) = value.parse::<usize>() {
                    base = Some(segment[..index].parse::<usize>().ok()?);
                    offset = Some(parsed);
                    break;
                }
            }
        }

        let base_value = base?;
        let offset_value = offset?;
        if total_lines == 0 {
            return None;
        }

        let start = match segment.chars().find(|ch| *ch == 'B' || *ch == 'C') {
            Some('B') => (base_value.saturating_sub(offset_value)).max(1),
            Some('C') => (base_value.saturating_sub(offset_value)).max(1),
            _ => base_value,
        };
        let end = match segment.chars().find(|ch| *ch == 'A' || *ch == 'C') {
            Some('A') => (base_value.saturating_add(offset_value)).min(total_lines),
            Some('C') => (base_value.saturating_add(offset_value)).min(total_lines),
            _ => base_value,
        };

        if start <= end {
            return Some(Range::new(start, end));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{CheckLineRange, Range};
    use crate::base::{CheckCondition, LineStatus};

    #[test]
    fn parses_basic_ranges() {
        let checker = CheckLineRange::with_total_lines("5..10,20".to_string(), 30);
        assert_eq!(checker.ranges, vec![Range::new(5, 10)]);
        assert!(checker.hashset.contains(&20));
    }

    #[test]
    fn parses_open_ended_range_from_start_line_upward() {
        let checker = CheckLineRange::with_total_lines("30..".to_string(), 0);
        let on_boundary = LineStatus { line_number: 30, ..Default::default() };
        let after_boundary = LineStatus { line_number: 31, ..Default::default() };
        let before_boundary = LineStatus { line_number: 29, ..Default::default() };

        assert_eq!(checker.ranges, vec![Range::new(30, usize::MAX)]);
        assert!(checker.check(&on_boundary));
        assert!(checker.check(&after_boundary));
        assert!(!checker.check(&before_boundary));
    }

    #[test]
    fn parses_head_tail_and_open_ranges() {
        let checker = CheckLineRange::with_total_lines("..5,h2,t3".to_string(), 12);
        assert_eq!(checker.ranges.len(), 3);
        assert!(checker.ranges.iter().any(|r| r == &Range::new(1, 5)));
        assert!(checker.ranges.iter().any(|r| r == &Range::new(1, 2)));
        assert!(checker.ranges.iter().any(|r| r == &Range::new(10, 12)));
    }

    #[test]
    fn parses_relative_negative_ranges() {
        let checker = CheckLineRange::with_total_lines("-5..,-5..-2".to_string(), 12);
        assert!(checker.ranges.iter().any(|r| r == &Range::new(8, 12)));
        assert!(checker.ranges.iter().any(|r| r == &Range::new(8, 11)));
    }

    #[test]
    fn deferred_total_lines_resolves_tail_and_negative_indices() {
        let tail_checker = CheckLineRange::new("t5".to_string());
        let negative_checker = CheckLineRange::new("-1".to_string());

        let line8 = LineStatus { line_number: 8, ..Default::default() };
        let line7 = LineStatus { line_number: 7, ..Default::default() };
        let line12 = LineStatus { line_number: 12, ..Default::default() };
        let line11 = LineStatus { line_number: 11, ..Default::default() };

        assert!(tail_checker.check_with_total_lines(&line8, 12));
        assert!(!tail_checker.check_with_total_lines(&line7, 12));
        assert!(negative_checker.check_with_total_lines(&line12, 12));
        assert!(!negative_checker.check_with_total_lines(&line11, 12));
    }

    #[test]
    fn parses_offset_ranges() {
        let checker = CheckLineRange::with_total_lines("4C2,4A2,4B2".to_string(), 12);
        assert!(checker.ranges.iter().any(|r| r == &Range::new(2, 6)));
        assert!(checker.ranges.iter().any(|r| r == &Range::new(4, 6)));
        assert!(checker.ranges.iter().any(|r| r == &Range::new(2, 4)));
    }

    #[test]
    fn checker_matches_range_and_hashset_values() {
        let checker = CheckLineRange::with_total_lines("4..6,1,3".to_string(), 10);
        let line1 = LineStatus { line_number: 1, ..Default::default() };
        let line5 = LineStatus { line_number: 5, ..Default::default() };
        let line8 = LineStatus { line_number: 8, ..Default::default() };

        assert!(checker.check(&line1));
        assert!(checker.check(&line5));
        assert!(!checker.check(&line8));
    }

    #[test]
    fn invalid_specs_are_noops() {
        let checker = CheckLineRange::with_total_lines("A..1,5..3".to_string(), 10);
        let any = LineStatus { line_number: 5, ..Default::default() };
        assert!(!checker.check(&any));
    }
}
