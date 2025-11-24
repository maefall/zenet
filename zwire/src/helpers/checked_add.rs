use crate::WireError;

pub trait CheckedAddWire: Sized {
    fn checked_add_wire(
        self,
        field_name_left_side: &'static str,
        right_side: Self,
        field_name_right_side: &'static str,
    ) -> Result<Self, WireError>;
}

impl CheckedAddWire for usize {
    fn checked_add_wire(
        self,
        field_name_left_side: &'static str,
        right_side: Self,
        field_name_right_side: &'static str,
    ) -> Result<Self, WireError> {
        self.checked_add(right_side)
            .ok_or(WireError::ArithmeticOverflow(
                self,
                field_name_left_side,
                right_side,
                field_name_right_side,
            ))
    }
}
