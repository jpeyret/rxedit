# Config

rxedit reads user configuration from:

~/.config/rxedit/config.toml

The file is optional. If it is missing, rxedit uses built-in defaults.

Generate a starter file with:

`rxedit --gen-config`

If the file already exists, it is not overwritten.

Supported keys (all optional, boolean values):

- `spacer`
- `line_number`
- `grep_shortcodes`

Example `config.toml`:

```toml
spacer = false
line_number = true
grep_shortcodes = true
```

Environment variable overrides (all optional):

- `rxedit_spacer`
- `rxedit_line_number`
- `rxedit_grep_shortcodes`

Language plugin directory precedence:

- `rxedit_plugins_directory`
- `[plugins].directory` in `config.toml`

Accepted boolean values for env vars:

- true forms: `true`, `1`, `yes`, `on`
- false forms: `false`, `0`, `no`, `off`
