# Status

## Syntax and Semantic Checks

### Completed

- SYNTAX (done during scanning)
- REVAR (have `check_for_redeclare` in `symbol_table.rs`)
- UNINIT (have `has_been_initialized` in `symbol_table.rs` - thought it may have to not take in span and use the symbol found?)

## In progress

- CONV (kinda started writing the visitor in `conv.rs`)

### Todo

- NOVAR
- EXPR
- UNUSED
- CONST