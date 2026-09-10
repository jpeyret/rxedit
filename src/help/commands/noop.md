# No-operation / noop

This isn't a command, but rather it is what happens when rxedit can't figure out what to do.

Rather than erroring out, it will just "do nothing" and pass on the lines to the next command.

- `explain` will try to tell why you ended up with No-operation.

- If the only command present is a No-operation, rxedit will ask you to specify at least one valid 
command.  

- Be careful when you are using rxedit to modify a file with `change` or `delete`.  A preceding
No-operation may mean that you are not modifying just the lines you intended to.
