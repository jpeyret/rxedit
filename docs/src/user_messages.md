
## User messages

|Key                                         | Code         | Body|
|--------------------------------------------| -------------| ----|
|declarations_hashtree_key_not_found         | RXE-ERR-0001 | hashtree_key={} not found|
                                                             
This error means a [`declarations`](./commands/declarations.md) hit referenced a tree-sitter key that was not found in the parsed declarations map.
This usually indicates declarations state got out of sync between parsing and filtering.


declarations_showowner_extension_unsupported  RXE-WARN-0001  apply:showowner extension did not match supported languages
                                                             
This warning means the programming language extension for the provided file does not have an rxedit tree-sitter grammar defined yet.
Basic commands like more/all/less still work, but syntax-aware features such as declarations and show owner are unavailable for that extension.


treesitter_name_field_missing                 RXE-WARN-0002  Could not find name field for node kind: {} : {}
                                                             
This warning means rxedit found a declarations node that does not expose the expected name field in the current grammar,
so that declarations is skipped when building the declaration index.


