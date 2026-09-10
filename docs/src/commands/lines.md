# Command `lines` 

Show lines within the given ranges.  Similar to `head - <number>` or `tail - <number>`

### Example:  show lines between 2 and 5, inclusive

` rxedit pysample.py lines::2-5 -n`

#### Output


```
000002 from pathlib import Path
000003 
000004 
000005 def do_init(func):
```

### More examples:

- `rxedit pysample.py  lines::-5,30-` show lines 1-5 and 30 to the end of the file

- `rxedit pysample.py  lines::1,3,5` show lines 1,3 and 5


## Lines flags

|Name     | Format             | Example        | Description|
|---------| -------------------| ---------------| -----------|
|set key  | sk=([a-z]+)(?:\.\|$|  ::sk=important|  set key to a value that can be used with wk=<key>. in later commands|
|where_key| wk=([a-z]+)(?:\.\|$|  ::wk=mykey.   |  puts an extra criteria to other commands|
