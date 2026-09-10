//! Primarily concerned with displaying Command-level feedback to the user via the `explain` command.
//! LineStatus is displayed mostly "as is"
//! it uses minijinja because the display for a command is non-trivial:
//! it depends on the Command's flags and on what the user actually entered

use crate::commands::search::Searcher;
use crate::common::{CheckKey, CheckLineRange, DeclarationsCommandQualifier, GrepCommandQualifier};
use crate::constants::{DEBUGGING, command_prefix, commandflags};
use crate::telemetry::{NoGrammarNotification, TelemetryEvent};
use core::fmt;
use minijinja::{Environment, context};
use serde::Serialize;

#[derive(Serialize, Debug)]
/// Template-ready view model for explain command output.
pub struct CommandExplainRepresentation {
    name: String,
    arg: String,
    unrecognized: String,
    searcher: String,
    lines: Vec<String>,
}

static INDENT_COMMAND_DETAILS: &str = "\n     ";
static INDENT_OUTCOMES: &str = "\n       ";
static NOOP_LABEL: &str = "no-operation";

fn write_noop_message(f: &mut fmt::Formatter<'_>, received: &str, cause: &str) -> fmt::Result {
    write!(
        f,
        "\n{} => [{}] so buffer passthrough.  Caused by: {}.  \n\n",
        received, NOOP_LABEL, cause
    )
}

fn build_templates() -> Environment<'static> {
    let mut env = Environment::new();
    env.add_template(command_prefix::EXPLAIN, include_str!("./declare.jinja"))
        .unwrap();
    env
}

impl fmt::Display for CheckLineRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let default = CheckLineRange::default();
        let mut tmp: Vec<String> = Vec::new();
        if self.until_ != default.until_ {
            tmp.push(format!("1-{}", self.until_));
        }
        if !self.wanted.is_empty() {
            tmp.push(format!("wanted {:?}", self.wanted));
        }
        if self.from_ != default.from_ {
            tmp.push(format!("{}-", self.from_));
        }
        if !tmp.is_empty() {
            tmp.insert(0, "lines ".to_string());
        }
        let res = join_indents(tmp, "/ ");
        write!(f, "{}", res)
    }
}

impl fmt::Display for GrepCommandQualifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags: Vec<String> = Vec::new();

        if self.before > 0 {
            flags.push(format!("-B {}", self.before));
        }

        if self.after > 0 {
            flags.push(format!("-A {}", self.after));
        }

        let res: String = if !flags.is_empty() {
            format!("\"grep\": {}", flags.join(" "))
        } else {
            "".to_string()
        };

        let mut tmp = Vec::new();
        tmp.push(res);

        let mut tmp2 = Vec::new();

        if self.showowner {
            tmp2.push(commandflags::SHOW_OWNER.name.to_string());
        }

        if self.user_confirmation {
            tmp2.push(commandflags::USER_CONFIRM.name.to_string());
        }

        if self.preformat {
            tmp2.push(commandflags::USE_GREP_SHORTCODES.name.to_string());
        }

        if !tmp2.is_empty() {
            let flags = join_indents(tmp2, " ");
            tmp.push(flags);
        }

        let mut wheres: Vec<String> = Vec::new();

        // handling wheres
        for b_cond in &self.conditions_checker.conds {
            if let Some(ck) = b_cond.as_any().downcast_ref::<CheckKey>() {
                // dbg!(&ck.required_key);
                wheres.push(format!("where key = `{}`", ck.required_key));
                continue;
            } else if let Some(clines) = b_cond.as_any().downcast_ref::<CheckLineRange>() {
                let (until_, wanted, from_) = (clines.until_, clines.wanted.clone(), clines.from_);

                let default = CheckLineRange::default();

                let mut tmp3 = Vec::new();
                if until_ != default.until_ {
                    tmp3.push(format!("1-{}", until_));
                };

                if !wanted.is_empty() {
                    tmp3.push(format!(" in {:?}", wanted));
                };

                if from_ != default.from_ {
                    // dbg!(&from_, default.from_);
                    tmp3.push(format!("{}-...", from_));
                };

                if !tmp3.is_empty() {
                    tmp3.insert(0, "lines:".to_string());
                    let s_ = join_indents(tmp3, " ");
                    wheres.push(s_);
                }
                continue;
            }
        }

        if !wheres.is_empty() {
            let s_ = join_indents(wheres, ",");
            tmp.push(s_);
        }

        let mut outcomes = Vec::new();

        if let Some(value) = &self.set_key {
            outcomes.push(format!("set key = `{}`", value));
        }

        if !outcomes.is_empty() {
            tmp.push("--- effects --- :".to_string());
        }

        let mut res = join_indents(tmp, INDENT_COMMAND_DETAILS);

        if !outcomes.is_empty() {
            outcomes.insert(0, " ".to_string());

            let tmp = join_indents(outcomes, INDENT_OUTCOMES);
            res.push_str(&tmp);
        }

        write!(f, "{}", res)
    }
}

// assemble non-empty lines for indent
fn join_indents(lines: Vec<String>, sep: &str) -> String {
    //howto- filter out empty strings
    let lines: Vec<String> = lines.into_iter().filter(|s| !s.is_empty()).collect();

    if !lines.is_empty() {
        lines.join(sep)
    } else {
        "".to_string()
    }
}

impl fmt::Display for DeclarationsCommandQualifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *DEBUGGING {
            dbg!(&self);
        }

        let mut tmp = Vec::new();
        tmp.push(self.searchqualifier.to_string());
        let mut tmp2 = Vec::new();

        if let Some(parents) = &self.parents {
            let mut tmp_parents = vec!["parents:".to_string()];
            for v in parents {
                tmp_parents.push(format!("{v}/"));
            }
            let s_ = tmp_parents.join(" ");

            tmp.push(s_);
        }

        if self.showbody {
            tmp2.push(commandflags::SHOW_BODY.name.to_string());
        }
        let flags = join_indents(tmp2, " ");

        tmp.push(flags);
        let res = join_indents(tmp, INDENT_COMMAND_DETAILS);
        write!(f, "{}", res)
    }
}

impl fmt::Display for Searcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *DEBUGGING {
            dbg!("\n\nSearcher=", &self);
        }

        let s_out = match &self {
            Searcher::RegexSearcher(cond) => {
                format!("regex: `{}`", cond.patre)
            }
            Searcher::RegexKeepSkip(cond) => {
                format!("regex: `{}` but not `{}`", cond.rekeep, cond.reskip)
            }
            Searcher::ContainsSearcher(cond) => {
                format!(
                    "contains: `{}` case sensitive: {}",
                    cond.pattern, !cond.insensitive
                )
            }
            Searcher::ContainsKeepSkip(cond) => {
                format!(
                    "contains: `{}` but not `{}` case sensitive: {}",
                    cond.keep, cond.skip, !cond.insensitive
                )
            }
        };
        write!(f, "{s_out}",)
    }
}

impl fmt::Display for NoGrammarNotification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cause = format!("extension `{}` is not a supported grammar", self.extension);
        write_noop_message(f, &self.payload, &cause)
    }
}

fn fmt_unrecognized(unrecognized: impl AsRef<str>) -> String {
    if unrecognized.as_ref().is_empty() {
        String::new()
    } else {
        format!(" /?ignored: `{}` ", unrecognized.as_ref())
    }
}

fn get_explain(
    name: impl AsRef<str>,
    arg: impl AsRef<str>,
    qualifier: impl AsRef<str>,
    unrecognized: impl AsRef<str>,
    lines: Vec<String>,
    searcher: Option<String>,
) -> String {
    let name: &str = name.as_ref();
    let arg: &str = arg.as_ref();
    let qualifier: &str = qualifier.as_ref();
    let unrecognized = unrecognized.as_ref();

    if *DEBUGGING {
        dbg!(&qualifier);
    }

    let env = build_templates();
    let template = env.get_template(command_prefix::EXPLAIN).unwrap();
    let unrecognized = fmt_unrecognized(unrecognized);
    if *DEBUGGING {
        dbg!("get_explain", &arg, &unrecognized);
    }
    let searcher = searcher.unwrap_or("".to_string());
    let command = CommandExplainRepresentation {
        name: name.to_owned(),
        arg: arg.to_owned(),
        unrecognized: unrecognized.to_owned(),
        searcher,
        lines,
    };
    format!("{}\n", template.render(context!(command)).unwrap())
}

impl fmt::Display for TelemetryEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TelemetryEvent::NoopNotification {
                payload: _,
                received,
                cause,
            } => write_noop_message(f, received, cause),
            TelemetryEvent::GenericCommandNotification {
                name: source,
                arg: payload,
                qualifier,
                unrecognized,
                searcher,
            } => {
                let qualifier = qualifier.to_owned();
                let unrecognized = unrecognized.to_owned();

                let mut lines = Vec::new();

                if *DEBUGGING {
                    dbg!(&qualifier);
                }
                lines.push(qualifier.to_string());

                write!(
                    f,
                    "{}",
                    get_explain(
                        source,
                        payload,
                        qualifier,
                        unrecognized,
                        lines,
                        Some(searcher.to_string())
                    )
                )
            }
            TelemetryEvent::NoGrammarNotification(notification) => {
                write!(f, "{notification}")
            }
            TelemetryEvent::ExplainNotification {
                source: _,
                payload: _,
            } => {
                write!(f, "")
            }
            TelemetryEvent::DeclarationsNotification {
                arg: payload,
                pattern: _,
                flags: _,
                extension: _,
                qualifier,
                unrecognized,
                // where_type,
                searcher,
            } => {
                let mut lines = Vec::new();

                if *DEBUGGING {
                    dbg!(&qualifier);
                }
                lines.push(qualifier.to_string());
                write!(
                    f,
                    "{}",
                    get_explain(
                        "declare",
                        payload,
                        qualifier,
                        unrecognized,
                        lines,
                        Some(searcher.to_owned())
                    )
                )
            }
        }
    }
}
