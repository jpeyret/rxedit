# Supported languages

Out of the box, Rust and Python are supported.  Adding a new language consists of finding a crate that delivers a 
`librxedit_language_<extension>` shared library and then symlinking to it in a directory indicated by environment variable `$rxedit_plugins_directory` or specified in 

##### ~/.config/rxedit/config.toml

```
[plugins]
directory = "~/.config/rxedit/plugins"

[tree_sitter.extensions]
sh = "zsh"
```

Here we tell it two things, the location of the plugins directory and also that `.sh` files should use the `zsh` parser.

Detecting the language is automatic, based on the file extension.  If the language is not supported, 
most `rxedit` commands will continue as before, but specific options, 
like the `::o` "show owner", flag will be ignored.  Basically, it falls back to doing anything 
grep can do, but treats the file as just text as it has no understanding of code syntax.

`declarations` commands will be entirely ignored however, as an `explain` will show.
