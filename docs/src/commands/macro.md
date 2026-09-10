# Command `macro` : 

Allows the execution of saved commands in succession against a target.

### Example:

Suppose you want to find bits of your code that are "chatty", perhaps with log calls 
or print statement and similar things put in during development.

You might start out with 

`rxedit pysample.py m::print m::pp\(`

but you don't want to flag lines that were already commented out so

`rxedit pysample.py m::print m::pp\( 'less::^ *#%'`

but now you also have to retype all of this on each new file you are scanning.  That's
where macro files come in, which allow you to save those commands for re-use and add comments:

So you can run this instead:

`rxedit pysample.py macro::./rxedit/showpychatty.rxi`

giving

```
000035         print(f"\n{self.name}'s History:")
000037             print(f"  {entry}")
```

then hide line 37...

`rxedit pysample.py macro::./rxedit/showpychatty.rxi  less::::wl=37`

```
000035         print(f"\n{self.name}'s History:")
```

and call another macro file to comment visible lines out.

`rxedit pysample.py macro::./rxedit/showpychatty.rxi  less::::wl=37 macro::./rxedit/pycommentout.rxi`

```
000035         #print(f"\n{self.name}'s History:")
```

### Sample macro file contents

Comments start with `#` and empty lines are ignored.

#### file `rxedit/showpychatty.rxi`

```
#things that we consider "noisy"
m::^ *(print|debug|pp|ppp|rin|rindebug)\(
more::log\.(info|debug)
more::log\(..level="w
#hide commented-out stuff
less::^ *#
```


#### file ` rxedit/pycommentout.rxi`

Note that rxedit's `Change` is just a passthrough to the standard Rust Regex crate which supports the use of
groups.

````
# put all leading space in match group1, rest in group2.  then prefix group2 with a `#`
'c::([\s]*)([^\s].*$)::$1#$2'
````

