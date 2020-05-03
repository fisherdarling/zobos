pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

pub struct Symbol {
    scope: usize,
    ty: String,    // ty = type. Should probably not be a string, but don't have types yet
    ident: String, // identifier
}

impl Symbol {
    pub fn new(scope: usize, ty: String, ident: String) -> Self {
        Self { scope, ty, ident }
    }
}
