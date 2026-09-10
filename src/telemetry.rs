//! Shared telemetry records for command activity and explain requests.
//! This module tracks events, no-grammar notices, and request counts.
//! It supports gathering data for the use of the `explain` command, how rxedit understood
//! user requests and, importantly, when it did not understand.
//! Each Command calls `append_telemetry` to register its arguments.

//! See [`crate::commands::explain`] for details about explain.

use std::sync::RwLock;

use once_cell::sync::Lazy;

/// Re-exported declarations and grep qualifiers used by telemetry events.
pub use crate::base::{DeclarationsCommandQualifier, GrepCommandQualifier};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Details for a telemetry event where no grammar is available.
pub struct NoGrammarNotification {
    /// File extension that had no grammar support.
    pub extension: String,
    /// Command source that triggered the lookup.
    pub source: String,
    /// Original command payload.
    pub payload: String,
    /// Parsed flags from the command.
    pub flags: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Telemetry event variants captured during command handling.
pub enum TelemetryEvent {
    /// Records a command that could not be understood.
    NoopNotification {
        /// Original payload text.
        payload: String,
        /// Exact received command text.
        received: String,
        /// Reason the command was treated as no-op.
        cause: String,
    },
    /// Records an explain command request.
    ExplainNotification {
        /// Command source.
        source: String,
        /// Original command payload.
        payload: String,
    },
    /// Records a recognized generic command.
    GenericCommandNotification {
        /// Command name.
        name: String,
        /// Original command payload.
        arg: String,
        /// Parsed qualifier value.
        qualifier: String,
        /// Searcher mode used by the command.
        searcher: String,
        /// Unrecognized command suffix.
        unrecognized: String,
    },
    /// Records a no-grammar notification.
    NoGrammarNotification(NoGrammarNotification),
    /// Records a declarations command parse result.
    DeclarationsNotification {
        /// Original command payload.
        arg: String,
        /// Extracted declarations pattern.
        pattern: String,
        /// Parsed flags from the command.
        flags: String,
        /// File extension used for grammar lookup.
        extension: String,
        /// Parsed declarations qualifier.
        qualifier: String,
        /// Searcher mode used by the command.
        searcher: String,
        /// Unrecognized command suffix.
        unrecognized: String,
        // where_type: Option<String>,
    },
}

#[derive(Debug, Clone, Default)]
/// In-memory telemetry state for the current process.
pub struct AppTelemetryState {
    /// Collected telemetry events.
    pub events: Vec<TelemetryEvent>,
    /// Count of explain requests since last drain.
    pub explain_requests: usize,
}

static TELEMETRY_STATE: Lazy<RwLock<AppTelemetryState>> =
    Lazy::new(|| RwLock::new(AppTelemetryState::default()));

/// Each Command calls this to notify the registry of how
/// it was built.
pub fn append_telemetry(event: TelemetryEvent) {
    let mut state = TELEMETRY_STATE
        .write()
        .expect("global telemetry state lock poisoned");
    state.events.push(event);
}

/// Returns a cloned snapshot of collected telemetry events.
pub fn telemetry_events() -> Vec<TelemetryEvent> {
    TELEMETRY_STATE
        .read()
        .expect("global telemetry state lock poisoned")
        .events
        .clone()
}

/// Clears collected events and resets explain request count.
pub fn clear_telemetry() {
    let mut state = TELEMETRY_STATE
        .write()
        .expect("global telemetry state lock poisoned");
    state.events.clear();
    state.explain_requests = 0;
}

/// Increments the number of pending explain requests.
pub fn request_explain() {
    let mut state = TELEMETRY_STATE
        .write()
        .expect("global telemetry state lock poisoned");
    state.explain_requests += 1;
}

/// Returns and resets the pending explain request count.
pub fn take_explain_requests() -> usize {
    let mut state = TELEMETRY_STATE
        .write()
        .expect("global telemetry state lock poisoned");
    let count = state.explain_requests;
    state.explain_requests = 0;
    count
}

#[cfg(test)]
mod tests {
    use super::{NoGrammarNotification, TelemetryEvent, clear_telemetry, telemetry_events};
    use once_cell::sync::Lazy;
    use std::sync::Mutex;

    use crate::common::with_global_config;

    static TEST_GLOBAL_CONFIG_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn set_test_global_config(file_path: &str) {
        with_global_config(|global| {
            global.file_path = file_path.to_string();
            global.grep_shortcodes = false;
            global.verbose = false;
            global.debug = false;
        });
    }

    fn lock_test_config() -> std::sync::MutexGuard<'static, ()> {
        TEST_GLOBAL_CONFIG_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn make_command_unrecognized_appends_telemetry() {
        let _guard = lock_test_config();
        set_test_global_config("sample.rs");
        clear_telemetry();

        let _ = crate::make_command("not-a-command");

        let events = telemetry_events();
        assert!(events.iter().any(|event| {
            *event
                == TelemetryEvent::NoopNotification {
                    payload: "not-a-command".to_string(),
                    received: "not-a-command".to_string(),
                    cause: "not understood".to_string(),
                }
        }));
    }


    #[test]
    fn make_command_more_appends_generic_command_notification() {
        let _guard = lock_test_config();
        set_test_global_config("sample.rs");
        clear_telemetry();

        let cmd = crate::make_command("more::alpha::B2");
        assert!(matches!(cmd, crate::Command::CMore(_)));

        let events = telemetry_events();
        let maybe_event = events.into_iter().find(|event| match event {
            TelemetryEvent::GenericCommandNotification {
                name: source,
                arg: payload,
                ..
            } => source == "more" && payload == "more::alpha::B2",
            _ => false,
        });

        let Some(TelemetryEvent::GenericCommandNotification {
            qualifier: _,
            unrecognized,
            ..
        }) = maybe_event
        else {
            panic!("expected generic command telemetry event");
        };

        // assert!(qualifier.as_ref().is_some_and(|value| !value.is_empty()));
        assert_eq!(unrecognized, "");
    }

    #[test]
    #[ignore]
    fn make_command_invert_appends_generic_command_notification() {
        let _guard = lock_test_config();
        set_test_global_config("sample.rs");
        clear_telemetry();

        let cmd = crate::make_command("invert");
        assert!(matches!(cmd, crate::Command::CInvert(_)));

        let events = telemetry_events();
        assert!(events.into_iter().any(|event| {
            event
                == TelemetryEvent::GenericCommandNotification {
                    name: "invert".to_string(),
                    arg: "invert".to_string(),
                    qualifier: "".to_string(),
                    unrecognized: "".to_string(),
                    searcher: "".to_string(),
                }
        }));
    }

    #[test]
    fn from_declarations_unknown_extension_appends_telemetry() {
        let _guard = lock_test_config();
        set_test_global_config("sample.txt");
        clear_telemetry();

        let cmd =
            crate::commands::declarations::from_declarations("alpha", "wt=f", "d::alpha::wt=f");

        let crate::Command::CNoop(noop) = cmd else {
            panic!("expected CNoop");
        };
        assert_eq!(
            noop.message,
            "declarations:  no grammar found for extension=txt".to_string()
        );

        let events = telemetry_events();
        assert!(events.into_iter().any(|event| {
            event
                == TelemetryEvent::NoGrammarNotification(NoGrammarNotification {
                    extension: "txt".to_string(),
                    source: "declarations".to_string(),
                    payload: "d::alpha::wt=f".to_string(),
                    flags: "wt=f".to_string(),
                })
        }));
    }
}
