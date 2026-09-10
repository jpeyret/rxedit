# Command `imports` - [i]imports (supported languages only)

Find imports/uses/includes that match the search term.

#### Example: search for imports that match `co`, but not `std` or `regex`.

`rxedit utilities.rs i::co--std,regex`

#### Output:

```
use crate::common::{CheckLineRange, LineStatus};
use crate::constants::{DEBUGGING, RE_SPLIT_NEGATIVES_NAMES};
    use crate::common::ConditionResult;
    use crate::common::GetLine;
    use crate::common::StopOrAddChecker;
    use crate::constants::Direction;
    use crate::common::LineStatus;
    use crate::constants::Direction;
```

#### Example: search for imports "far in the file"

Another example if we want to look for imports that are past the reasonable start of the file, line 100 in 
the example below.
Sometimes however, with Python, this is done to avoid circular import dependencies, so we'll use the `::o`
"show owner" flag as well.

`rxedit utils.py i::::wl=100-.o`
