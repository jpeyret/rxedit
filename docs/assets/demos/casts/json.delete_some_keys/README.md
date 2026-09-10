## Use rxedit to delete json keys, taking care with trailing commas

Yes, `jq` would do this much more easily.  But this shows a toggle mode, if X do Y else do Z.  And rxedit doesn't need to know json syntax either.

##### remove_del.rxi

```
# this deletes all "del"  keys from an indent-formatted json.

#show all "del" entries and set_key them to `todelete`
all::^ *"del"::sk=todelete 

#we can delete the ones with a trailing comma
delete::,$:: 

#now show the lines before "del" keys as well
all::::B1wk=todelete

#delete the targets
delete::::wk=todelete

#strip off the trailing comma
change::,$::
```

Note that we are using set/where key `todelete` to keep track of the rows we are interested in.

And `all::::B1wk=todelete` manages to show them again, but also fire off the B1 we need to trim off the preceding line's comma.

