use crate::WireError;

pub trait CheckedAddWire: Sized {
    fn checked_add_wire(
        self,
        right_side: Self,
        field_name: &'static str,
        field_name_right_side: &'static str,
    ) -> Result<Self, WireError>;
}

impl CheckedAddWire for usize {
    fn checked_add_wire(
        self,
        right_side: Self,
        field_name: &'static str,
        field_name_right_side: &'static str,
    ) -> Result<Self, WireError> {
        self.checked_add(right_side)
            .ok_or(WireError::ArithmethicOverflow(
                self,
                right_side,
                field_name,
                field_name_right_side,
            ))
    }
}
