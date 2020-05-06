use crate::ast::*;
use crate::hazards::*;
use crate::visitor::*;
pub struct Conversion;

impl Visitor for Conversion {
    fn visit_assign(&mut self, assign: &AstNode) {
        assert_eq!(AstKind::Assign, assign.kind);

        let left_type: String = Visitor::get_type(self, &assign[0]);
        let right_type: String = Visitor::get_type(self, &assign[1]);

        if is_valid_conversion(&left_type, &right_type) {
            let h = Hazard::new_one_loc(
                HazardType::ErrorT(ErrorId::Conversion),
                assign.span.0,
                assign.span.1,
            );
            println!("{}", h.show_output());
        }
    }
}

pub fn is_valid_conversion(lhs: &str, rhs: &str) -> bool {
    match lhs {
        "string" => match rhs {
            "int" => false,
            "float" => false,
            "bool" => false,
            _ => true,
        },
        "float" => match rhs {
            "int" => false,
            "bool" => false,
            "string" => false,
            _ => true,
        },
        "bool" => match rhs {
            "float" => false,
            "string" => false,
            _ => true,
        },
        "int" => match rhs {
            "bool" => false,
            "string" => false,
            _ => true,
        },
        _ => panic!("unknown lhs passed into valid conversion"),
    }
}
// asdfs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversions() {
        assert!(is_valid_conversion("int", "int"));
        assert!(is_valid_conversion("int", "float"));
        assert!(!is_valid_conversion("string", "int"));
        assert!(!is_valid_conversion("bool", "string"));
    }
}
