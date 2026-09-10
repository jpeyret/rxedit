# Why chaining commands is so powerful.

That's rxedit's superpower, otherwise it would be a less capable `grep` or `sed`. 
The idea is to make lines selectively visible by either adding to, or hiding, lines that were
made visible by previous commands.

Except for [`more`](./commands/more.md) and [`all`](./commands/all.md) only visible lines are affected by any commands.  This applies 
specifically to [`delete`](./commands/delete.md) and [`change`](./commands/change.md).  [`invert`](./commands/invert.md) can be used to "flip" the visibility.

For example, to extract only lines of interest to a report (using the `-o <filename> command line flag), 
you would:

`rxedit myjava.java all::public invert delete -o myreport.out`

- show [`all`](./commands/all.md) lines you care about
- optionally, apply an appropriate [`less`](./commands/less.md) to remove commented out lines or 
- [`invert`](./commands/invert.md) the visibility
- [`delete`](./commands/delete.md) everything
- `-o myreport.txt` would then save only lines with lines of interest.  
  - (No, you certainly wouldn't want want to use the `-O` modify in place option here).

You could also search for lines that indicate debugging code, hide some of the ones you want to keep
and then run a macro with predefined language-specific [`change`](./commands/change.md) commands to comment out those debugging lines.


​                                                                                                                          
###  Workflow Help                                                                                                           

  1) Start with broad match commands to reveal relevant lines, possibly using existing macro files.
  2) Narrow progressively using additional commands.
  3) Add -n for line numbers and -s for visual spacing.
  4) Make sure your visible lines match your expections.

#### If modifying the file

  5) If modifying the file with `prepend/append`, [`change`](./commands/change.md) or [`delete`](./commands/delete.md), make doubly sure the results
  are as expected.
  6) When satisfied, use -O to modify the file in place (best to do it on a version-controlled directory)

### SEE ALSO 

`rxedit --help change` shows a fairly complex chaining example to comment "print" statements.

