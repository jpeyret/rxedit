use crate::base::{CheckCondition, ConditionContext, LineStatus};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
/// stores lines of interest,
pub struct Range {
    /// the start of the range
    pub start: usize,
    /// the end of the range
    pub end: usize,
}

impl Range {
    /// create a from .. to range, inclusive
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// checks if a linenum is within the range
    pub fn contains(&self, value: usize) -> bool {
        self.start <= value && value <= self.end
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// these store user requests relative to the end, which is not known at parse time.
pub struct DeferredRange {
    /// the start of the range, which can be positive (from the head) or negative from the tail
    pub start: i32,
    /// it's an Option because -2.. is bound to the end, not a particular offset
    pub end: Option<i32>,
}

impl std::fmt::Display for DeferredRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let end = match self.end {
            Some(value) => value.to_string(),
            None => "end".to_string(),
        };

        write!(f, "{}..{}", self.start, end)
    }
}

/// end-based Range, ex: -2.. means the last 2 lines, but we don't what the total linecount
/// is so it is deferred till apply is called.
impl DeferredRange {
    /// create a new, end-based Range
    pub fn new(start: i32, end: Option<i32>) -> Self {
        Self { start, end }
    }

    /// once we know the `Vec<LineStatus>` size, we can calculate negative offsets
    pub fn resolve(&self, total_lines: usize) -> Option<Range> {
        if total_lines == 0 {
            return None;
        }

        let start = resolve_index(self.start, total_lines)?;
        let end = match self.end {
            Some(value) => resolve_index(value, total_lines)?,
            None => total_lines,
        };

        if start <= end {
            Some(Range::new(start, end))
        } else {
            None
        }
    }
}

/// tracks user line numbers criteria
#[derive(Debug, Clone)]
pub struct CheckLineRange {
    /// New range-based representation for the DSL.
    pub where_linenum: String,
    /// from..to line numbers allowed
    pub ranges: Vec<Range>,
    /// from..to line numbers allowed, but relative to the end, via negative offsets
    pub deferred_end_ranges: Vec<DeferredRange>,
    /// discrete linenums to allow
    pub hashset: HashSet<usize>,
    /// discrete linenums to allow, but as negative offsets from the end
    pub deferred_end: HashSet<usize>,
    /// what to report in the `explain` telemetry string
    pub for_telemetry: String,
}

/// a CheckLineRange allows the user to specify bounds on the line locations to
/// consider.
impl CheckLineRange {
    /// create a new CheckLineRange from user specs
    pub fn new(where_linenum: String) -> Self {
        Self::with_total_lines(where_linenum, 0)
    }

    /// Validates that every comma-separated segment is syntactically understood by the line DSL.
    pub fn validate_spec(where_linenum: &str) -> Result<(), String> {
        for raw_segment in where_linenum.split(',') {
            let segment = raw_segment.trim();
            if segment.is_empty() {
                continue;
            }

            if is_valid_segment_syntax(segment) {
                continue;
            }

            return Err(format!("invalid line range segment `{segment}`"));
        }

        Ok(())
    }

    /// builds a CheckLineRange, possibly with deferred Ranges and indices when counting from the end
    pub fn with_total_lines(where_linenum: String, total_lines: usize) -> Self {
        let mut ranges = Vec::new();
        let mut deferred_end_ranges = Vec::new();
        let mut hashset = HashSet::new();
        let mut deferred_end = HashSet::new();

        if where_linenum.trim().is_empty() {
            return Self {
                where_linenum,
                ranges,
                deferred_end_ranges,
                hashset,
                deferred_end,
                for_telemetry: "".to_string(),
            };
        }

        // build its telemetry here, which will match user entry sequence
        let mut temp_telem: Vec<String> = Vec::new();

        for raw_segment in where_linenum.split(',') {
            let segment = raw_segment.trim();
            if segment.is_empty() {
                continue;
            }

            match parse_segment(segment) {
                ParsedSegment::Range(range) => {
                    temp_telem.push(format!("{}", range));
                    ranges.push(range)
                }
                ParsedSegment::DeferredRange(range) => {
                    temp_telem.push(format!("{}", range));
                    deferred_end_ranges.push(range)
                }
                ParsedSegment::Line(value) => {
                    temp_telem.push(value.to_string());
                    hashset.insert(value);
                }
                ParsedSegment::DeferredLine(offset) => {
                    temp_telem.push(format!("-{}", offset));
                    deferred_end.insert(offset);
                }
                ParsedSegment::None => {}
            }
        }

        //overlapping ranges get collapsed, discrete point in ranges as well...
        normalize_concrete(&mut ranges, &mut hashset);

        let for_telemetry: String = temp_telem.join(", ");

        let base = Self {
            where_linenum,
            ranges,
            deferred_end_ranges,
            hashset,
            deferred_end,
            for_telemetry,
        };

        if total_lines > 0 {
            base.materialize(total_lines)
        } else {
            base
        }
    }

    fn materialized_concrete(&self, total_lines: usize) -> (Vec<Range>, HashSet<usize>) {
        let mut ranges = self.ranges.clone();
        let mut hashset = self.hashset.clone();

        for deferred in &self.deferred_end_ranges {
            if let Some(range) = deferred.resolve(total_lines) {
                ranges.push(range);
            }
        }

        for offset in &self.deferred_end {
            if let Some(value) = resolve_negative_offset(*offset, total_lines) {
                hashset.insert(value);
            }
        }

        normalize_concrete(&mut ranges, &mut hashset);
        (ranges, hashset)
    }

    fn materialize(&self, total_lines: usize) -> Self {
        let (ranges, hashset) = self.materialized_concrete(total_lines);
        Self {
            where_linenum: self.where_linenum.clone(),
            ranges,
            deferred_end_ranges: self.deferred_end_ranges.clone(),
            hashset,
            deferred_end: self.deferred_end.clone(),
            for_telemetry: self.for_telemetry.clone(),
        }
    }
}

impl Default for CheckLineRange {
    fn default() -> Self {
        Self {
            where_linenum: String::new(),
            ranges: Vec::new(),
            deferred_end_ranges: Vec::new(),
            hashset: HashSet::new(),
            deferred_end: HashSet::new(),
            for_telemetry: "???".to_string(),
        }
    }
}

impl CheckCondition for CheckLineRange {
    fn check(&self, line: &LineStatus) -> bool {
        self.ranges
            .iter()
            .any(|range| range.contains(line.line_number))
            || self.hashset.contains(&line.line_number)
    }

    fn check_with_context(&self, line: &LineStatus, context: &ConditionContext) -> bool {
        let total_lines = context.total_lines.unwrap_or(0);
        if self.where_linenum.is_empty() || total_lines == 0 {
            return self.check(line);
        }

        let (ranges, hashset) = self.materialized_concrete(total_lines);
        ranges.iter().any(|range| range.contains(line.line_number))
            || hashset.contains(&line.line_number)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

enum ParsedSegment {
    Range(Range),
    DeferredRange(DeferredRange),
    Line(usize),
    DeferredLine(usize),
    None,
}

enum ParsedRangeSegment {
    Concrete(Range),
    Deferred(DeferredRange),
}

fn parse_segment(segment: &str) -> ParsedSegment {
    let segment = segment.trim();

    if let Some(range) = parse_range_segment(segment) {
        return match range {
            ParsedRangeSegment::Concrete(range) => ParsedSegment::Range(range),
            ParsedRangeSegment::Deferred(range) => ParsedSegment::DeferredRange(range),
        };
    }

    if let Ok(value) = segment.parse::<usize>() {
        return ParsedSegment::Line(value);
    }

    if segment.starts_with('-')
        && segment.len() > 1
        && let Ok(offset) = segment[1..].parse::<usize>()
        && offset > 0
    {
        return ParsedSegment::DeferredLine(offset);
    }

    ParsedSegment::None
}

fn parse_range_segment(segment: &str) -> Option<ParsedRangeSegment> {
    if segment.contains("..") {
        let mut parts = segment.split("..");
        let left = parts.next().unwrap_or("");
        let right = parts.next().unwrap_or("");
        if parts.next().is_some() {
            return None;
        }

        let start = if left.is_empty() {
            1
        } else {
            parse_signed_i32(left)?
        };
        let end = if right.is_empty() {
            None
        } else {
            Some(parse_signed_i32(right)?)
        };

        let needs_deferred = start < 0 || end.is_some_and(|value| value < 0);
        if needs_deferred {
            if let Some(end_value) = end
                && start < 0
                && end_value < 0
                && start > end_value
            {
                return None;
            }
            return Some(ParsedRangeSegment::Deferred(DeferredRange::new(start, end)));
        }

        let start_u = usize::try_from(start).ok()?;
        let end_u = if let Some(end_value) = end {
            usize::try_from(end_value).ok()?
        } else {
            usize::MAX
        };

        if start_u <= end_u {
            return Some(ParsedRangeSegment::Concrete(Range::new(start_u, end_u)));
        }
        return None;
    }

    if segment.starts_with("h") && segment.len() > 1 {
        let count = segment[1..].parse::<usize>().ok()?;
        return Some(ParsedRangeSegment::Concrete(Range::new(1, count)));
    }
    if segment == "h" {
        return Some(ParsedRangeSegment::Concrete(Range::new(1, 10)));
    }
    if segment.starts_with("t") && segment.len() > 1 {
        let count = segment[1..].parse::<usize>().ok()?;
        return Some(ParsedRangeSegment::Deferred(DeferredRange::new(
            -(count as i32),
            None,
        )));
    }
    if segment == "t" {
        return Some(ParsedRangeSegment::Deferred(DeferredRange::new(-10, None)));
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

        let start = match segment.chars().find(|ch| *ch == 'B' || *ch == 'C') {
            Some('B') => (base_value.saturating_sub(offset_value)).max(1),
            Some('C') => (base_value.saturating_sub(offset_value)).max(1),
            _ => base_value,
        };
        let end = match segment.chars().find(|ch| *ch == 'A' || *ch == 'C') {
            Some('A') => base_value.saturating_add(offset_value),
            Some('C') => base_value.saturating_add(offset_value),
            _ => base_value,
        };

        if start <= end {
            return Some(ParsedRangeSegment::Concrete(Range::new(start, end)));
        }
    }

    None
}

fn parse_signed_i32(raw: &str) -> Option<i32> {
    raw.parse::<i32>().ok()
}

fn resolve_index(value: i32, total_lines: usize) -> Option<usize> {
    if value == 0 {
        return None;
    }
    if value > 0 {
        return usize::try_from(value).ok();
    }

    let offset = usize::try_from(value.unsigned_abs()).ok()?;
    if offset == 0 || offset > total_lines {
        return None;
    }
    Some(total_lines.saturating_sub(offset).saturating_add(1))
}

fn resolve_negative_offset(offset: usize, total_lines: usize) -> Option<usize> {
    if offset == 0 || offset > total_lines {
        return None;
    }
    Some(total_lines.saturating_sub(offset).saturating_add(1))
}

fn normalize_concrete(ranges: &mut Vec<Range>, hashset: &mut HashSet<usize>) {
    ranges.sort_by_key(|range| range.start);
    ranges.dedup_by(|current, next| current.start == next.start && current.end == next.end);

    let mut filtered_hashset = HashSet::new();
    for value in hashset.iter().copied() {
        let covered = ranges.iter().any(|range| range.contains(value));
        if !covered {
            filtered_hashset.insert(value);
        }
    }

    *hashset = filtered_hashset;
}

fn is_valid_segment_syntax(segment: &str) -> bool {
    let segment = segment.trim();

    if segment.is_empty() {
        return true;
    }

    if segment.contains("..") {
        let mut parts = segment.split("..");
        let left = parts.next().unwrap_or("");
        let right = parts.next().unwrap_or("");
        if parts.next().is_some() {
            return false;
        }

        return is_bound_syntax_valid(left) && is_bound_syntax_valid(right);
    }

    if segment.starts_with('h') || segment.starts_with('t') {
        return segment == "h" || segment == "t" || segment[1..].parse::<usize>().is_ok();
    }

    if segment.contains('A') || segment.contains('B') || segment.contains('C') {
        for marker in ['A', 'B', 'C'] {
            if let Some(index) = segment.find(marker) {
                let left = &segment[..index];
                let right = &segment[index + 1..];
                return left.parse::<usize>().is_ok() && right.parse::<usize>().is_ok();
            }
        }
        return false;
    }

    segment.parse::<usize>().is_ok()
        || (segment.starts_with('-') && segment.len() > 1 && segment[1..].parse::<usize>().is_ok())
}

fn is_bound_syntax_valid(raw: &str) -> bool {
    if raw.is_empty() {
        return true;
    }

    raw.parse::<usize>().is_ok()
        || (raw.starts_with('-') && raw.len() > 1 && raw[1..].parse::<usize>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::{CheckLineRange, DeferredRange, Range};
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
        let on_boundary = LineStatus {
            line_number: 30,
            ..Default::default()
        };
        let after_boundary = LineStatus {
            line_number: 31,
            ..Default::default()
        };
        let before_boundary = LineStatus {
            line_number: 29,
            ..Default::default()
        };

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
    fn stores_deferred_ranges_without_total_lines() {
        let checker = CheckLineRange::new("-5..,-5..-2".to_string());
        assert!(checker.ranges.is_empty());
        assert!(
            checker
                .deferred_end_ranges
                .iter()
                .any(|r| r == &DeferredRange::new(-5, None))
        );
        assert!(
            checker
                .deferred_end_ranges
                .iter()
                .any(|r| r == &DeferredRange::new(-5, Some(-2)))
        );
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
        let line1 = LineStatus {
            line_number: 1,
            ..Default::default()
        };
        let line5 = LineStatus {
            line_number: 5,
            ..Default::default()
        };
        let line8 = LineStatus {
            line_number: 8,
            ..Default::default()
        };

        assert!(checker.check(&line1));
        assert!(checker.check(&line5));
        assert!(!checker.check(&line8));
    }

    #[test]
    fn invalid_specs_are_noops() {
        let checker = CheckLineRange::with_total_lines("A..1,5..3".to_string(), 10);
        let any = LineStatus {
            line_number: 5,
            ..Default::default()
        };
        assert!(!checker.check(&any));
    }

    #[test]
    fn validate_spec_rejects_invalid_segments() {
        let error = CheckLineRange::validate_spec("foobar..3").expect_err("expected invalid spec");
        assert_eq!(error, "invalid line range segment `foobar..3`");
    }

    #[test]
    fn validate_spec_accepts_deferred_tail_ranges() {
        CheckLineRange::validate_spec("..2,-2..").expect("expected valid spec");
    }
}
