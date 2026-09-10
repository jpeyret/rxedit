//! Responsible for parsing the flags payload to results appropriate for each
//! Command, and also tracking what was not understood as a flag, for later
//! use by `explain`.
//! Each flag can either be a boolean or have some data attached to it
//! The `A` flag for example has to be followed by 1 or more digits.
//! As each flag is found, it is removed from the payload and the remainder
//! is returned.

use crate::common::CheckConditions;
use crate::common::{CheckKey, CheckLineRange, GrepCommandQualifier, get_global_config};
use regex::Regex;
use std::io::{self, IsTerminal};
use std::process;

use crate::constants::FlagDef;
use crate::constants::commandflags;
use crate::loader_for_constants;
use once_cell::sync::Lazy;

static AFTER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(loader_for_constants::resolve(
        "commandflags.AFTER_CONTEXT.value",
    ))
    .expect("AFTER_CONTEXT regex must be valid")
});

static SETKEY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(commandflags::SETKEY.value).expect("SETKEY regex must be valid"));

static WHEREKEY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(commandflags::WHEREKEY.value).expect("WHEREKEY regex must be valid"));

static WHERE_LINENUM_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(commandflags::WHERE_LINENUM.value).expect("WHERE_LINENUM_REGEX regex must be valid")
});

/// Regex used to parse the `wt=` where-type filter flag.
pub static WHERETYPE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(commandflags::WHERETYPE.value).expect("WHERETYPE regex must be valid"));

static BEFORE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(loader_for_constants::resolve(
        "commandflags.BEFORE_CONTEXT.value",
    ))
    .expect("BEFORE_CONTEXT regex must be valid")
});

static CONTEXT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(commandflags::CONTEXT.value).expect("CONTEXT regex must be valid"));

impl GrepCommandQualifier {
    /// Builds a qualifier from flags using the default search flag set.
    pub fn build(argstring: &str) -> (GrepCommandQualifier, String) {
        Self::build_with_flags(argstring, commandflags::searchflags())
    }

    /// returns a GrepCommandQualifier, possibly with all-defaults values
    /// and the remainder of what was not consumed by it from argstring
    /// ex: build("A5XYZ") => GrepCommandQualifier(after:5, ...defaults), "XYZ"
    pub fn build_with_flags(
        argstring: &str,
        allowed_flags: &'static [FlagDef],
    ) -> (GrepCommandQualifier, String) {
        let global_config = get_global_config();

        let allow_setkey = allows_flag(allowed_flags, commandflags::SETKEY.name);
        let allow_after = allows_flag(allowed_flags, commandflags::AFTER_CONTEXT.name);
        let allow_before = allows_flag(allowed_flags, commandflags::BEFORE_CONTEXT.name);
        let allow_context = allows_flag(allowed_flags, commandflags::CONTEXT.name);
        let allow_fixed = allows_flag(allowed_flags, commandflags::FIXED_STRING.name);
        let allow_case_insensitive =
            allows_flag(allowed_flags, commandflags::CASE_INSENSITIVE.name);
        let allow_preformat = allows_flag(allowed_flags, commandflags::USE_GREP_SHORTCODES.name);
        let allow_wherekey = allows_flag(allowed_flags, commandflags::WHEREKEY.name);
        let allow_showowner = allows_flag(allowed_flags, commandflags::SHOW_OWNER.name);

        let allow_where_linenum = allows_flag(allowed_flags, commandflags::WHERE_LINENUM.name);

        let allow_user_confirm = allows_flag(allowed_flags, commandflags::USER_CONFIRM.name);
        let (remaining, after_match) = if allow_after {
            return_match_and_consume(&AFTER_REGEX, argstring)
        } else {
            (argstring.to_string(), "".to_string())
        };

        let mut conditions_checker = CheckConditions::new();

        let (remaining, where_linenum) = if allow_where_linenum {
            return_capture_and_consume(&WHERE_LINENUM_REGEX, &remaining)
        } else {
            (remaining, None)
        };

        if let Some(where_linenum) = where_linenum {
            let checker = CheckLineRange::new(where_linenum);
            conditions_checker.add_condition(checker);
        }

        let (remaining, before_match) = if allow_before {
            return_match_and_consume(&BEFORE_REGEX, &remaining)
        } else {
            (remaining, "".to_string())
        };
        let (remaining, context_match) = if allow_context {
            return_match_and_consume(&CONTEXT_REGEX, &remaining)
        } else {
            (remaining, "".to_string())
        };

        // sk= is a multi-char token (sk=<word>;); parse it before single-char flag
        // passes so that a key containing 'i', 'F', etc. is not mangled.

        let (remaining, set_key) = if allow_setkey {
            return_capture_and_consume(&SETKEY_REGEX, &remaining)
        } else {
            (remaining, None)
        };
        let (remaining, where_key) = if allow_wherekey {
            return_capture_and_consume(&WHEREKEY_REGEX, &remaining)
        } else {
            (remaining, None)
        };

        if let Some(key) = where_key {
            conditions_checker.add_condition(CheckKey::new(key));
        }

        let context_counter: i32 = context_match
            .strip_prefix('C')
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let after_counter: i32 = after_match
            .strip_prefix('A')
            .and_then(|s| s.parse().ok())
            .unwrap_or(context_counter);

        let before_counter: i32 = before_match
            .strip_prefix('B')
            .and_then(|s| s.parse().ok())
            .unwrap_or(context_counter);

        let (remaining, fixed_string) = if allow_fixed {
            return_contains_and_consume(commandflags::FIXED_STRING.value, &remaining)
        } else {
            (remaining, false)
        };

        let (remaining, case_insensitive) = if allow_case_insensitive {
            return_contains_and_consume(commandflags::CASE_INSENSITIVE.value, &remaining)
        } else {
            (remaining, false)
        };

        let (remaining, user_confirmation) = if allow_user_confirm {
            let (remaining, user_confirmation) =
                return_contains_and_consume(commandflags::USER_CONFIRM.value, &remaining);
            if user_confirmation {
                // can't allow request for user confirmation from within a batch, that wouldn't work
                if !io::stdin().is_terminal() {
                    eprintln!("user confirmation only supported in interactive mode");
                    process::exit(1);
                }
            }
            (remaining, user_confirmation)
        } else {
            (remaining, false)
        };

        // going into this, you have the global config, then 2 command-leval booleans, both off by default:
        // command_use_shortcodes and command_no_use_shortcodes
        // if either command booleans are true, use the command branch, else use the global config
        let (remaining, preformat) = if allow_preformat {
            let (remaining, command_use_shortcodes) =
                return_contains_and_consume(commandflags::NO_GREP_SHORTCODES.value, &remaining);
            let (remaining, command_no_use_shortcodes) =
                return_contains_and_consume(commandflags::USE_GREP_SHORTCODES.value, &remaining);

            let shortcodes: bool = if command_use_shortcodes || command_no_use_shortcodes {
                // local, command-level, branch.  true if use is true and not_use is false
                command_no_use_shortcodes && !command_use_shortcodes
            } else {
                global_config.grep_shortcodes
            };

            (remaining, shortcodes)
        } else {
            (remaining, false)
        };

        let (remaining, showowner) = if allow_showowner {
            return_contains_and_consume(commandflags::SHOW_OWNER.value, &remaining)
        } else {
            (remaining, false)
        };

        let preformat = preformat && !fixed_string;

        let regex_flags = if case_insensitive {
            Some("i".to_string())
        } else {
            None
        };

        let unconsumed = remaining;

        let qualifier: GrepCommandQualifier = GrepCommandQualifier {
            after: after_counter,
            before: before_counter,
            preformat,
            fixed_string,
            regex_flags,
            set_key,
            showowner,
            user_confirmation,
            unrecognized: unconsumed.clone(),
            conditions_checker,
        };

        (qualifier, unconsumed)
    }
}

fn allows_flag(allowed_flags: &'static [FlagDef], flag_name: &str) -> bool {
    allowed_flags.iter().any(|f| f.name == flag_name)
}

/// called by a command on a flag by flag basis, it looks for the presence of the flag
/// and its payload.  if it finds it takes out the flags content and returns it value
fn return_match_and_consume(patre: &Lazy<Regex>, haystack: &str) -> (String, String) {
    // if the regex matches return its Capture and take it out of the haystack
    // regex of A(\\d+) , A5B123 should return `("A5","B123")`

    match patre.find(haystack) {
        None => (haystack.to_string(), "".to_string()),
        Some(m) => {
            let matched = m.as_str().to_string();
            let unconsumed = format!("{}{}", &haystack[..m.start()], &haystack[m.end()..]);
            (unconsumed, matched)
        }
    }
}

/// called by a command on a flag by flag basis for boolean flags
/// if it finds it takes the flag character from the "flags payload" and returns the remainder and true
pub fn return_contains_and_consume(pat: &str, haystack: &str) -> (String, bool) {
    let found = haystack.contains(pat);
    let remain = haystack.replace(pat, "");
    (remain, found)
}

/// If the regex matches, consume the full match from haystack and return capture group 1.
/// E.g. `sk=exc.` with pattern `sk=([a-z]+)\.` returns `(remaining, Some("exc"))`.
pub fn return_capture_and_consume(patre: &Lazy<Regex>, haystack: &str) -> (String, Option<String>) {
    match patre.captures(haystack) {
        None => (haystack.to_string(), None),
        Some(caps) => {
            let m = caps.get(0).unwrap();
            let captured = caps.get(1).map(|c| c.as_str().to_string());
            let unconsumed = format!("{}{}", &haystack[..m.start()], &haystack[m.end()..]);
            (unconsumed, captured)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::flag_utilities::AFTER_REGEX;

    use super::return_match_and_consume;
    use crate::common::{GrepCommandQualifier, LineStatus, MetaInfo};
    use crate::constants::commandflags;

    fn make_line_with_key(key: &str) -> LineStatus {
        LineStatus {
            meta: MetaInfo {
                key: key.to_string(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    #[test]
    fn return_match_and_consume_no_match() {
        let tosearch = "iFG";
        let (unconsumed, match_) = return_match_and_consume(&AFTER_REGEX, tosearch);

        assert_eq!(unconsumed, tosearch);
        assert_eq!(match_, "");
    }

    #[test]
    fn build_parses_after_and_before_counters() {
        let (qualifier, unconsumed) = GrepCommandQualifier::build("A3B2XY");
        assert_eq!(qualifier.after, 3);
        assert_eq!(qualifier.before, 2);
        assert_eq!(unconsumed, "XY");
    }

    #[test]
    fn build_parses_context_counter_for_both_sides() {
        let (qualifier, unconsumed) = GrepCommandQualifier::build("C4XY");
        assert_eq!(qualifier.after, 4);
        assert_eq!(qualifier.before, 4);
        assert_eq!(unconsumed, "XY");
    }

    #[test]
    fn build_parses_insentive_to_regexflags() {
        let (qualifier, unconsumed) = GrepCommandQualifier::build("C4XYi");
        assert_eq!(qualifier.after, 4);
        assert_eq!(qualifier.before, 4);
        assert_eq!(unconsumed, "XY");
        assert_eq!(qualifier.regex_flags, Some("i".to_string()));
    }

    #[test]
    fn build_parses_fixed_string_flag() {
        let (qualifier, unconsumed) = GrepCommandQualifier::build("FXY");
        assert!(qualifier.fixed_string);
        assert_eq!(unconsumed, "XY");
    }

    #[test]
    fn build_defaults_when_empty() {
        let (qualifier, unconsumed) = GrepCommandQualifier::build("");
        assert_eq!(qualifier.after, 0);
        assert_eq!(qualifier.before, 0);
        assert!(!qualifier.fixed_string);
        assert_eq!(unconsumed, "");
    }

    #[test]
    fn build_unconsumed_passthrough() {
        let (qualifier, unconsumed) = GrepCommandQualifier::build("iXYZ");
        assert_eq!(qualifier.after, 0);
        assert_eq!(qualifier.before, 0);
        // 'i' is a regex flag, not a command flag — passes through unconsumed
        assert_eq!(unconsumed, "XYZ");
    }

    #[test]
    fn build_with_delete_flags_does_not_consume_context_flags() {
        let (qualifier, unconsumed) =
            GrepCommandQualifier::build_with_flags("XY", commandflags::delete_flags());
        assert_eq!(qualifier.after, 0);
        assert_eq!(qualifier.before, 0);
        assert_eq!("XY", unconsumed);
    }

    #[test]
    fn build_with_delete_flags_still_consumes_supported_flags() {
        let (qualifier, unconsumed) =
            GrepCommandQualifier::build_with_flags("FiNXY", commandflags::delete_flags());
        assert!(qualifier.fixed_string);
        // assert!(qualifier.suppress_line_counters);
        assert!(!qualifier.showowner);
        assert_eq!(qualifier.regex_flags, Some("i".to_string()));
        assert_eq!("NXY", unconsumed);
    }

    #[test]
    fn build_parses_showowner_flag() {
        let (qualifier, unconsumed) = GrepCommandQualifier::build("oXY");
        assert!(qualifier.showowner);
        assert_eq!(unconsumed, "XY");
    }

    #[test]
    fn build_parses_setkey_flag() {
        let (qualifier, unconsumed) = GrepCommandQualifier::build("sk=exc.XY");
        assert_eq!(qualifier.set_key, Some("exc".to_string()));
        assert_eq!(unconsumed, "XY");
    }

    #[test]
    fn build_parses_setkey_flag_at_end_of_string() {
        let (qualifier, unconsumed) = GrepCommandQualifier::build("sk=exc");
        assert_eq!(qualifier.set_key, Some("exc".to_string()));
        assert_eq!(unconsumed, "");
    }

    #[test]
    fn build_setkey_absent_gives_none() {
        let (qualifier, _) = GrepCommandQualifier::build("iXY");
        assert_eq!(qualifier.set_key, None);
    }

    #[test]
    fn build_parses_wherekey_flag() {
        let (qualifier, unconsumed) = GrepCommandQualifier::build("wk=priv.XY");
        let matching = make_line_with_key("priv");
        let non_matching = make_line_with_key("other");
        assert!(qualifier.conditions_checker.check(&matching));
        assert!(!qualifier.conditions_checker.check(&non_matching));
        assert_eq!(unconsumed, "XY");
    }

    #[test]
    fn build_parses_wherekey_flag_at_end_of_string() {
        let (qualifier, unconsumed) = GrepCommandQualifier::build("wk=priv");
        let matching = make_line_with_key("priv");
        assert!(qualifier.conditions_checker.check(&matching));
        assert_eq!(unconsumed, "");
    }

    #[test]
    fn build_with_delete_flags_parses_wherekey_flag() {
        let (qualifier, unconsumed) =
            GrepCommandQualifier::build_with_flags("wk=priv.XY", commandflags::delete_flags());
        let matching = make_line_with_key("priv");
        let non_matching = make_line_with_key("other");
        assert!(qualifier.conditions_checker.check(&matching));
        assert!(!qualifier.conditions_checker.check(&non_matching));
        assert_eq!(unconsumed, "XY");
    }

    #[test]
    fn return_match_and_consume_match() {
        let tosearch = "iFA10G";
        let (unconsumed, match_) = return_match_and_consume(&AFTER_REGEX, tosearch);

        assert_eq!(unconsumed, "iFG");
        assert_eq!(match_, "A10");
    }
}
