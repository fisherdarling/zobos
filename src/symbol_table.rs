pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn output(&self) -> String {
        let mut out = String::new();

        for s in &self.symbols {
            out.push_str(&s.output());
            out.push('\n')
        }
        out
    }
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
    pub fn output(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.scope.to_string());
        out.push(',');
        out.push_str(&self.ty.to_string());
        out.push(',');
        out.push_str(&self.ident.to_string());
        out.clone().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_table_output() {
        let s1 = Symbol::new(0, String::from("test_type"), String::from("test_ident"));
        let s2 = Symbol::new(
            0,
            String::from("test_type_var_2"),
            String::from("test_ident_var_2"),
        );
        assert_eq!(s1.output(), "0,test_type,test_ident");
        let syms = vec![s1, s2];
        let table = SymbolTable { symbols: syms };
        let output = table.output();
        assert_eq!(
            output,
            r#"0,test_type,test_ident
0,test_type_var_2,test_ident_var_2
"#
        );
    }
}
