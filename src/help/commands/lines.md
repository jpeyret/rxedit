# Command `lines` 

Show lines within the given ranges.  Similar to `head - <number>` or `tail - <number>`, with a number of possible specifier.



### Example:  show lines between 2 and 5, inclusive

` rxedit pysample.py lines::2..5 -n`

#### Output


```
000002 from pathlib import Path
000003 
000004 
000005 def do_init(func):
```

#### Supported format, for `lines` and as used in the `wl=` flag.

- `x..y` means a from x - to y  range.  Omitting x means "from the start", omitting y means "to the end".  x has to be <= y.
- all numbers are 1-based, inclusive
- `,` separates different groups, `/` indicates the end of the specifier (when used as a flag)
- negative indexing means count from the end, with -1 being the last line, -2 next to last, similar to Python's use
- `h` and `t` act like `head` and `tail` command, while `A`, `B`, `C` behave like they do in `grep` 


| Specifier     | Meaning                                |
| ------------- | -------------------------------------- |
| lines::2..5   | skip line 1, then display up to line 5 |
| lines::-1     | last line                              |
| lines::t5     | last 5 lines                           |
| lines::-5..   | last 5 lines                           |
| lines::h3     | first 3 lines                          |
| lines::-5..-3 | same as tail -5, but skip the last 2   |
| lines::30C5   | display lines 25-35                    |
| lines::..2,t5 | first 2, last 5                        |
| lines::..     | all lines                              |

#### Sample use of line ranges in other commands:

`rxedit pysample.py all::DEF::wl=30C5i`

Note that it wasn't necessary to terminate the flag with `/` because this flag always ends with `.` or a digit anyway.  So `i` meant case-insensitive.

#### Output

```
000031     def show_history(self) -> None:
```



### More examples:

- `rxedit pysample.py  lines::h5,30..` show lines 1-5 and 30 to the end of the file

- `rxedit pysample.py  lines::1,3,5` show lines 1,3 and 5



