# Status

## Syntax and Semantic Checks

### Completed

- SYNTAX (done during scanning)
- REVAR (have `check_for_redeclare` in `symbol_table.rs`) // Fisher: I think we got this?
- UNINIT (have `has_been_initialized` in `symbol_table.rs` - thought it may have to not take in span and use the symbol found?)
- UNUSED // Fisher: We now report this in main:
- NOVAR  // Fisher: We now report this in identifier resolution (get_expr_type base case).

## In progress

- CONV (kinda started writing the visitor in `conv.rs`)
- EXPR // Fisher: How close are we? Needs testing though kinda works?
- CONST // Fisher: Haven't really check for this since we need to implement
        // assignment nodes that are not declarations

### Todo

- Traversing the entire tree
- Emitting and reporting symbol tables