//! Utility functions to deal with Vec of source code lines.
use crate::common::LineStatus;
use crate::constants::{DEBUGGING, RE_SPLIT_NEGATIVES_NAMES};
use regex::Regex;
use std::fs;

/// holds functions to operate on Vec.LineStatus
pub mod vec_lines {
    use super::LineStatus;

    use crate::common::GetLine;

    use crate::constants::Direction;
    use sha2::{Digest, Sha256};
    use std::collections::HashSet;

    /// iterate sorted indices from the start and/or end and remove exterior empty/blank lines
    pub fn trim_outer_blanks(
        lines: &[LineStatus],
        indices: HashSet<usize>,
        sides: Direction,
    ) -> HashSet<usize> {
        let mut skip = HashSet::new();

        // remove after
        if sides == Direction::TRAILING || sides == Direction::BOTH {
            let mut li: Vec<usize> = indices.iter().copied().collect();
            li.sort_by(|a, b| b.cmp(a));

            for ix in li {
                if lines[ix].line.trim() != "" {
                    break;
                } else {
                    skip.insert(ix);
                };
            }
        }
        // remove before
        if sides == Direction::LEADING || sides == Direction::BOTH {
            let mut li: Vec<usize> = indices.iter().copied().collect();
            li.sort();

            for ix in li {
                if lines[ix].line.trim() != "" {
                    break;
                } else {
                    skip.insert(ix);
                };
            }
        }

        indices.difference(&skip).copied().collect()
    }

    /// re-assemble all lines as if for the file system.
    pub fn vec_lines_to_text<T: GetLine>(v_lines: &[T]) -> String {
        v_lines
            .iter()
            .map(|ls| ls.line())
            .collect::<Vec<&str>>()
            .join("\n")
    }

    /// calculate sha 256 for all lines
    pub fn digest_lines<T: GetLine>(lines: &[T]) -> String {
        let mut hasher = Sha256::new();
        for item in lines {
            hasher.update(item.line().as_bytes());
            hasher.update(b"\n"); // preserves boundaries + order
        }
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    /// adds indices by radius in either direction around hits to extends, bounded by vec.len()
    pub fn bounded_extend<T>(extends: &mut HashSet<usize>, hits: &[usize], radius: i32, vec: &[T]) {
        if radius == 0 {
            return;
        }

        if radius < 0 {
            for hit in hits {
                let ihit = *hit as i32;
                let lim = std::cmp::max(ihit + radius, 0);
                for ix in lim..ihit {
                    extends.insert(ix as usize);
                }
            }
        } else {
            let size = vec.len() as i32;
            for hit in hits {
                let ihit = *hit as i32;
                let lim = std::cmp::min(ihit + radius, size - 1);
                for ix in ihit + 1..lim + 1 {
                    extends.insert(ix as usize);
                }
            }
        };
    }
}

/// splits for nested searches
/// ex:  Foo/__init__ looks for __init__ under a parent matching Foo
pub fn split_search_parents(value: &str) -> (String, Option<Vec<String>>) {
    let mut v: Vec<String> = value.split("/").map(|v| v.to_string()).collect();
    v.reverse();
    if v.len() == 1 {
        (v[0].clone(), None)
    } else {
        (v[0].clone(), Some(v[1..].to_vec()))
    }
}

/// this is used to split a name search into matching -- filter out parts
/// splits are supported on '--|!' so `Url!^_` and `Url--^_` both mean matches `Url` but doesn't star with with'_`
pub fn split_name_negation(arg1: &str) -> (&str, Option<&str>) {
    let v_split: Vec<&str> = RE_SPLIT_NEGATIVES_NAMES.splitn(arg1, 2).collect();
    if *DEBUGGING {
        eprintln!("072.315.split_name_negation:v2:{arg1} => {:?}", v_split);
    }

    let res = match v_split.as_slice() {
        [pos, neg] | [pos, neg, ..] => {
            let neg = neg.trim();
            let neg = if neg.is_empty() { None } else { Some(neg) };
            (pos.trim(), neg)
        }
        [pos1] => (pos1.trim(), None),
        _ => unreachable!("split always yields at least one segment"),
    };
    if *DEBUGGING {
        eprintln!("072.split_name_negation:arg1={arg1} => {:?}", res);
    }
    res
}

/// regex on names (functions, classes...) can support a negation component. this splits the payload
pub fn split_name_negation_regex(arg1: &str) -> (Regex, Option<Regex>) {
    let (pos, neg) = split_name_negation(arg1);
    let rneg: Option<Regex> =
        neg.map(|neg2| Regex::new(&neg2.replace(",", "|")).expect("invalid neg regex neg={neg}"));
    // !!!TODO!!! use the name shortcodes for expansion via lib::preformat .  propagate error up to generate a Noop
    (
        Regex::new(&pos.replace(",", "|")).expect("invalid pos regex pos={pos}"),
        rneg,
    )
}

/// Reads commands from a file, ignoring blank lines and `#` comments.
pub fn parse_commands_file(path: &str) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read commands file '{}': {}", path, e))?;

    let strip_quotes: &[_] = &['"', '\''];
    let mut commands = Vec::new();
    for raw_line in content.lines() {
        let raw_line = raw_line.trim_matches(strip_quotes);

        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        commands.push(line.to_string());
    }
    Ok(commands)
}

#[cfg(test)]
mod tests {

    use super::vec_lines::bounded_extend;
    use crate::common::LineStatus;
    use crate::constants::Direction;
    use crate::utilities::vec_lines::trim_outer_blanks;
    use std::collections::HashSet;

    fn ls(s: &str) -> LineStatus {
        LineStatus {
            line: s.to_string(),
            ..Default::default()
        }
    }

    fn build_bounded_args(size: usize, exp: &[usize]) -> (Vec<usize>, HashSet<usize>) {
        let mut res0 = Vec::new();
        for i in 0..size {
            res0.push(i);
        }

        let mut res = HashSet::new();
        for ix in exp {
            res.insert(*ix);
        }
        (res0, res)
    }

    #[test]
    fn t_bounded_extend_base_ante() {
        let (lines, exp) = build_bounded_args(6, &[1, 2]);

        let mut got = HashSet::new();

        bounded_extend(&mut got, &[0, 3], -2, &lines);
        assert_eq!(exp, got);
    }

    #[test]
    fn t_bounded_extend_base_post() {
        let (lines, exp) = build_bounded_args(6, &[1, 2, 4, 5]);

        let mut got = HashSet::new();

        bounded_extend(&mut got, &[0, 3], 2, &lines);

        assert_eq!(exp, got);
    }

    #[test]
    fn t_bounded_extend_ante_post() {
        let (lines, exp) = build_bounded_args(6, &[1, 2, 4, 5]);
        let mut got = HashSet::new();
        bounded_extend(&mut got, &[3], -2, &lines);
        bounded_extend(&mut got, &[3], 2, &lines);
        assert_eq!(exp, got);
    }

    #[test]
    fn trim_end_blanks_removes_trailing_blank_lines_from_indices() {
        let lines = vec![ls("keep"), ls(""), ls("   "), ls("\t")];
        let indices: HashSet<usize> = HashSet::from([0, 1, 2, 3]);

        let got = trim_outer_blanks(&lines, indices, Direction::TRAILING);

        let expected: HashSet<usize> = HashSet::from([0]);
        assert_eq!(got, expected);
    }

    #[test]
    fn trim_end_blanks_keeps_indices_when_last_selected_is_non_blank() {
        let lines = vec![ls("a"), ls(""), ls("b")];
        let indices: HashSet<usize> = HashSet::from([0, 1, 2]);

        let got = trim_outer_blanks(&lines, indices.clone(), Direction::TRAILING);

        assert_eq!(got, indices);
    }

    #[test]
    fn trim_end_blanks_stops_on_first_selected_non_blank_from_end() {
        let lines = vec![ls("a"), ls(""), ls("x"), ls("b"), ls("")];
        let indices: HashSet<usize> = HashSet::from([1, 3, 4]);

        let got = trim_outer_blanks(&lines, indices, Direction::TRAILING);

        let expected: HashSet<usize> = HashSet::from([1, 3]);
        assert_eq!(got, expected);
    }
}
