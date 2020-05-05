pub struct Hazard {
    hazard_type: HazardType,
    locations: Vec<Location>,
}

pub struct Location {
    row: usize,
    col: usize,
}
impl Location {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl Hazard {
    pub fn new(hazard_type: HazardType, locations: Vec<Location>) -> Self {
        Self {
            hazard_type,
            locations,
        }
    }

    /// useful if you only have one location so you don't have to init a vec
    pub fn new_one_loc(hazard_type: HazardType, row: usize, col: usize) -> Self {
        let l = Location::new(row, col);
        let mut locations = Vec::new();
        locations.push(l);
        Self {
            hazard_type,
            locations,
        }
    }

    pub fn show_output(&self) -> String {
        let mut out = String::from("OUTPUT :");
        out.push_str(self.hazard_type.display_type());
        out.push(':');
        out.push(' ');
        for loc in &self.locations {
            out.push_str(&loc.row.to_string());
            out.push(' ');
            out.push_str(&loc.col.to_string());
        }
        out.push(' ');
        out.push(':');
        out.push_str(self.hazard_type.display_id());
        out.push(':');
        out
    }
}

pub enum HazardType {
    Syntax,          // String should be token
    ErrorT(ErrorId), // `Error` and `Err` where causing match problem
    Warn(WarnId),
}
impl HazardType {
    pub fn display_type(&self) -> &str {
        match self {
            HazardType::Syntax => "SYNTAX",
            HazardType::ErrorT(_e) => "ERROR",
            HazardType::Warn(_w) => "WARN",
        }
    }
    pub fn display_id(&self) -> &str {
        match self {
            HazardType::Syntax => "SYNTAX",
            HazardType::ErrorT(e) => match e {
                ErrorId::NoVar => "NOVAR",
                ErrorId::Conversion => "CONV",
                ErrorId::Expr => "EXPR",
            },
            HazardType::Warn(w) => match w {
                WarnId::Const => "CONST",
                WarnId::RedeclareVar => "REVAR",
                WarnId::Unused => "UNUSED",
                WarnId::Uninit => "UNINIT",
            },
        }
    }
}

pub enum ErrorId {
    NoVar,      // undeclared var
    Conversion, // value conversion error
    Expr,       // Expression tree operand error
}
pub enum WarnId {
    RedeclareVar, // Attempting to re-declare a variable
    Unused,       // The variable is not used in an expression or assignment within it's scope
    Uninit,       // Using a variable in an expression before it has been initialized with a value
    Const,        // Attempting to store a value in a variable with the const attribute
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn display_test() {
        assert_eq!(
            "OUTPUT :SYNTAX: 7 1 :SYNTAX:",
            Hazard::new_one_loc(HazardType::Syntax, 7, 1).show_output()
        )
    }

    #[test]
    fn display_type_tests() {
        let s = HazardType::Syntax;
        let e = HazardType::ErrorT(ErrorId::Conversion);
        let w = HazardType::Warn(WarnId::Const);
        assert_eq!(s.display_type(), "SYNTAX");
        assert_eq!(e.display_type(), "ERROR");
        assert_eq!(w.display_type(), "WARN");
    }

    #[test]
    fn display_id() {
        let s = HazardType::Syntax;
        assert_eq!(s.display_id(), "SYNTAX");

        let novar = HazardType::ErrorT(ErrorId::NoVar);
        let conv = HazardType::ErrorT(ErrorId::Conversion);
        let expr = HazardType::ErrorT(ErrorId::Expr);

        assert_eq!(novar.display_id(), "NOVAR");
        assert_eq!(conv.display_id(), "CONV");
        assert_eq!(expr.display_id(), "EXPR");

        let revar = HazardType::Warn(WarnId::RedeclareVar);
        let unused = HazardType::Warn(WarnId::Unused);
        let uninit = HazardType::Warn(WarnId::Uninit);
        let constT = HazardType::Warn(WarnId::Const);

        assert_eq!(revar.display_id(), "REVAR");
        assert_eq!(unused.display_id(), "UNUSED");
        assert_eq!(uninit.display_id(), "UNINIT");
        assert_eq!(constT.display_id(), "CONST");
    }
}
