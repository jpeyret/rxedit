#### Sample of a zsh plugin to enable tree-sitter support on extra languages.

Note:  This is a quick point-in-time preview of how language support is planned/implemented, not a polished guide.

Expect this integration to evolve a bit in the near future.



### Plugin discovery is simply extension-based 

##### ~/.config/rxedit

Note that we can point extensions to particular parsers.

```
[plugins]
directory = "~/.config/rxedit/plugins"

[tree_sitter.extensions]
sh = "zsh"
```

And, yes, in line with other rxedit behavior it would be nice to allow override parser assignment on the commandline.

##### directory contents 

```
~/.config/rxedit
├── config.toml
└── plugins
    ├── librxedit_language_js.dylib  # these are both symlinks
    └── librxedit_language_zsh.dylib
```



At this point all a plugin has to do is implement the `language_api::LanguageHelper`trait.


#### Actual implementation example at `developer_doc/language_plugins/zsh/lib.rs`

librxedit_language_zsh.dylib is just symlinking to the compiled binary.