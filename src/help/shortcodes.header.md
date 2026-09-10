
# Grep macro shortcodes.
Shortcodes can be applied to all regex searches, using the `-g` flag to turn it on (`-G` turns shortcodes off ).
They facilitate some common regex use cases and avoid clashing with shell substitutions.

##### Examples:  
- `rxedit sample.rs 'm::impl.*for'` can be replaced with `rxedit -g sample.rs m::impl..for`.  Note that quotes aren't needed anymore.
- `rxedit sample.rs 'm::fn|struct'` with `rxedit sample.rs -g m::fn~~o.struct `

##### Available shortcuts:
