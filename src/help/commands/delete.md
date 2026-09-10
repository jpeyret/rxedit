# Command Delete - [del]ete

Deletes visible lines matching pattern.

## Example delete visible lines containing "debug"

`rxedit sample.rs a::debug delete:debug`


## Details

- `delete` removes lines from the output entirely (not just hiding them).
- use `less` or set key / where key (`sk=`,`wk=`) to constraint the scope.
