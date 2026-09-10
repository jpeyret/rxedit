## checklist on adding a typical Command:

(this is from the implementation of CLines)

##### commands/lines.rs

```
pub struct CLines{   // just as an example
    pub condition: common::CheckLineRange,
    pub qualifier: GrepCommandQualifier,
}
```

commands need to indicate if they mutate the source lines, like `delete::` for example.

````
impl CLines {  // command
    pub const MUTATES_LINE_TEXT: bool = false;
}
````

`from_arg`  returns both the command and the telemetry event used by `debrief::`

````
pub(crate) fn from_arg(arg: &str, payload: &str, flags: &str) -> (TelemetryEvent, Command) {
````



most implementations of `apply` are used to flip the `LineStatus.visible` bool, so they tend to use this as an auxiliary function.

`fn f_calculate_visibility(visible: bool, hit: bool) -> bool {`

`apply` is the primary function that commands are built to implement.  They take in a `Vec<LineStatus>` and return another one (or the same as is).

````
impl CommandActions for CLines {  
    fn apply(
        &self,
        lines: Vec<LineStatus>,
        _hashtree: &HashMap<usize, common::Parsed>,
    ) -> Vec<LineStatus> {

````

##### constants.rs

first give the long command_prefix for the command, and optionally a short one.  `lines` is what we are after here.

```
pub mod command_prefix{   
   ...
   pub const LINES: &str = "lines";
   ...
```



constants needs to indicate what flags the command supports.

````
pub mod commandflags{
    ...
    pub const LINESFLAGS: &[FlagDef] = &[
        NO_LINE_COUNTERS,
        YES_LINE_COUNTERS,
        SETKEY,
        WHEREKEY,
    ];
    ...
````

##### lib.rs

`make_command` needs to be adjusted to recognize the command_prefix.  with this particular command build, it returns its own telemetry event, so just append it here and return the command, which bypasses the need for generic telemetry management further down.

````
 pub fn make_command(arg: &str) -> Command {
  ...
  [command_prefix::LINES, arg0, flag] => {
    let (telemetry_event, command) = commands::lines::from_arg(arg,arg0, flag);
    append_telemetry(telemetry_event);
    return command;
  }
  [command_prefix::LINES, arg0] => {
    let (telemetry_event, command) = commands::lines::from_arg(arg, arg0, "");
    append_telemetry(telemetry_event);
    return command;
  }
````

