use core::fmt;

pub use crate::base::{CheckCondition, LineStatus};

/// checks if a LineStatus is visible
struct CheckVisible;
/// always true
struct CheckAlwaysTrue;

impl CheckCondition for CheckVisible {
    fn check(&self, line: &LineStatus) -> bool {
        line.visible
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Debug for CheckVisible {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CheckVisible")
    }
}

impl CheckCondition for CheckAlwaysTrue {
    fn check(&self, _line: &LineStatus) -> bool {
        true
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Debug for CheckAlwaysTrue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CheckAlwaysTrue")
    }
}

///filters what goes to output files
pub enum OutputChecker {
    /// report behavior - only write what's visible
    CheckVisible,
    /// default behavior - anything in the buffer is written to file
    CheckAlwaysTrue,
}

impl OutputChecker {
    ///check if a LineStatus should be written to file
    pub fn check(&self, line: &LineStatus) -> bool {
        match self {
            OutputChecker::CheckVisible => CheckVisible.check(line),
            OutputChecker::CheckAlwaysTrue => CheckAlwaysTrue.check(line),
        }
    }
}
