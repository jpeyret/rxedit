## Clean a specific recurring Python style warning in code.

Lots of code below gives ruff linter warnings about `e`  not getting used:

```
except (Exception,) as e:
    if some_condition:
      breakpoint()
```

needs to be transformed to: 

`except (Exception,) as _e:`

A predefined macro does the trick:

##### fix_exception_noise.rxi

Note that the macro files don't need surrounding quotes around the commands.  But they tolerate them just fine.

````

#show except ... as e: which ruff flags as unused F841
more::^ *except (\({0,1}.+\){0,1}) as e~~colon.

#hide it if has a `# noqa ` set already...
less::#.* *noqa

#Keep the except and the exception clause in regex named group $1 but change variable `e` to `_e` which ruff will ignore
#Note: `~~colon.` is the shortcode for `:` because `::` is the field separator for rxedit
change::except (\({0,1}.+\){0,1}) as e~~colon.::except $1 as _e:
````

Note that the `change` is using some arcane syntax `$1` which you will not find referenced in the docs here.  That's because it is simply using the syntax of the Rust regex crate for substitions, which it doesn't know anything about. Bad regex syntax?  That becomes a No-op which you can see with `explain`.

Could `sed` do this?  Yes.  But we are also taking care not to change lines that have the `# noqa` ignore directive for the ruff linter.

You can also preview the changes from this macro, which does not save any changes.  A simple `-O` added to the command line persists the change.  That's also typical here, explore the commands needed via the cli history, until everything is good, then `-O` to save.  Complicated, but re-usable?  Put it in a macro.
