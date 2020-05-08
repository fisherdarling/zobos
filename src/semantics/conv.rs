use crate::ast::*;
use crate::hazards::*;
use crate::visitor::*;
pub struct Conversion;

// impl Visitor for Conversion {
//     fn visit_assign(&mut self, assign: &AstNode) {
//         assert_eq!(AstKind::Assign, assign.kind);

//         let left_type: String = Visitor::get_type(self, &assign[0]);
//         let right_type: String = Visitor::get_type(self, &assign[1]);

//         if !is_valid_conversion(&left_type, &right_type) {
//             let h = Hazard::new_one_loc(
//                 HazardType::ErrorT(ErrorId::Conversion),
//                 assign.span.0,
//                 assign.span.1,
//             );
//             println!("{}", h.show_output());
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversions() {
        assert!(is_valid_conversion("int", "int"));
        assert!(is_valid_conversion("float", "int"));
        assert!(!is_valid_conversion("int", "string"));
        assert!(!is_valid_conversion("string", "bool"));
    }
}
