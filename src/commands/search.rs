use crate::commands::prelude::*;
use crate::constants as c;
use crate::{CAll, CAnd, CDelete, CLess, CMore, CNoop, Command, new_searcher};
use regex::Regex;

/// Search behavior shared by all searcher implementations.
pub trait Search {
    /// Returns true when the line matches this searcher.
    fn search(&self, line: &str) -> bool;

    // some cases need to look specifically if the skip was activated
    /// Returns true when the line matches the skip condition.
    fn skip(&self, _line: &str) -> bool {
        false
    }
}

#[derive(Debug, Clone)]
/// Regex-only searcher.
pub struct RegexSearcher {
    pub(crate) patre: Regex,
}

#[derive(Debug, Clone)]
/// Regex searcher with keep and skip patterns.
pub struct RegexKeepSkip {
    pub(crate) rekeep: Regex,
    pub(crate) reskip: Regex,
}

impl Search for RegexKeepSkip {
    fn search(&self, line: &str) -> bool {
        self.rekeep.is_match(line) && !self.reskip.is_match(line)
    }
    fn skip(&self, line: &str) -> bool {
        self.reskip.is_match(line)
    }
}

impl Search for RegexSearcher {
    fn search(&self, line: &str) -> bool {
        self.patre.is_match(line)
    }
}

#[derive(Debug, Clone)]
/// Substring searcher with optional case-insensitive matching.
pub struct ContainsSearcher {
    pub(crate) pattern: String,
    pub(crate) insensitive: bool,
}

impl ContainsSearcher {
    /// Creates a contains searcher from pattern and case option.
    pub fn new(pattern: String, insensitive: bool) -> Self {
        let mut pattern = pattern;

        if insensitive {
            pattern = pattern.to_lowercase();
        }

        ContainsSearcher {
            pattern,
            insensitive,
        }
    }
}

impl Search for ContainsSearcher {
    fn search(&self, line: &str) -> bool {
        if !self.insensitive {
            return line.contains(&self.pattern);
        }
        line.to_lowercase().contains(&self.pattern)
    }
}

#[derive(Debug, Clone)]
/// Substring searcher with keep and skip terms.
pub struct ContainsKeepSkip {
    pub(crate) keep: String,
    pub(crate) skip: String,
    pub(crate) insensitive: bool,
}

impl ContainsKeepSkip {
    /// Creates a keep/skip contains searcher.
    pub fn new(keep: String, skip: String, insensitive: bool) -> Self {
        let mut keep = keep;
        let mut skip = skip;

        if insensitive {
            keep = keep.to_lowercase();
            skip = skip.to_ascii_lowercase();
        }

        Self {
            keep,
            skip,
            insensitive,
        }
    }
}

impl Search for ContainsKeepSkip {
    fn search(&self, line: &str) -> bool {
        if !self.insensitive {
            return line.contains(&self.keep) && !line.contains(&self.skip);
        }
        let line = line.to_lowercase();
        line.contains(&self.keep) && !line.contains(&self.skip)
    }
    fn skip(&self, line: &str) -> bool {
        if !self.insensitive {
            return line.contains(&self.skip);
        }
        line.to_lowercase().contains(&self.skip)
    }
}

#[derive(Debug, Clone)]
/// Concrete searcher variants used by commands.
pub enum Searcher {
    /// Regex-only searcher variant.
    RegexSearcher(RegexSearcher),
    /// Contains-only searcher variant.
    ContainsSearcher(ContainsSearcher),
    /// Contains keep/skip searcher variant.
    ContainsKeepSkip(ContainsKeepSkip),
    /// Regex keep/skip searcher variant.
    RegexKeepSkip(RegexKeepSkip),
}

// use core::fmt;

impl Search for Searcher {
    fn search(&self, line: &str) -> bool {
        match self {
            Searcher::RegexSearcher(searcher) => searcher.search(line),
            Searcher::ContainsSearcher(searcher) => searcher.search(line),
            Searcher::ContainsKeepSkip(searcher) => searcher.search(line),
            Searcher::RegexKeepSkip(searcher) => searcher.search(line),
        }
    }
    fn skip(&self, line: &str) -> bool {
        match self {
            Searcher::RegexSearcher(searcher) => searcher.skip(line),
            Searcher::ContainsSearcher(searcher) => searcher.skip(line),
            Searcher::ContainsKeepSkip(searcher) => searcher.skip(line),
            Searcher::RegexKeepSkip(searcher) => searcher.skip(line),
        }
    }
}

pub(crate) fn from_search(
    variant: CommandVariant,
    pattern: &str,
    flags: &str,
    arg: &str,
) -> Command {
    let allowed_flags = match variant {
        CommandVariant::CDelete => c::commandflags::delete_flags(),
        CommandVariant::CAnd => c::commandflags::and_flags(),
        _ => c::commandflags::searchflags(),
    };
    let result = new_searcher(
        pattern,
        None,
        flags,
        allowed_flags,
        c::grep_shortcodes::searchfield(),
    );

    if let Ok(seed) = result {
        let qualifier =
            CommandQualifier::GrepQualifier(seed.qualifier.expect("missing grep qualifier"));

        match variant {
            CommandVariant::CAnd => Command::CAnd(CAnd {
                searcher: seed.searcher,
                qualifier,
            }),
            CommandVariant::CMore => Command::CMore(CMore {
                searcher: seed.searcher,
                qualifier,
            }),
            CommandVariant::CLess => Command::CLess(CLess {
                searcher: seed.searcher,
                qualifier,
            }),
            CommandVariant::CAll => Command::CAll(CAll {
                searcher: seed.searcher,
                qualifier,
            }),
            CommandVariant::CDelete => Command::CDelete(CDelete {
                searcher: seed.searcher,
                qualifier,
            }),
        }
    } else {
        use crate::common::{TelemetryEvent, append_telemetry};

        let error = result.expect_err("error must exist for failed searcher build");
        let error_message = error.to_string();

        append_telemetry(TelemetryEvent::NoopNotification {
            payload: pattern.to_string(),
            received: arg.to_string(),
            cause: error_message.clone(),
        });

        Command::CNoop(CNoop {
            text: arg.to_string(),
            message: error_message,
        })
    }
}
