use crate::WireError;

macro_rules! length_check {
    ($source:expr) => {
        if $source.len() < Self::SIZE {
            return None;
        }
    };
}

macro_rules! impl_to_bytes {
    () => {
        #[inline]
        fn to_bytes(value: Self::Int) -> Self::ByteArray {
            value.to_be_bytes()
        }
    };
}

macro_rules! impl_to_bytes_from_usize {
    () => {
        #[inline]
        fn to_bytes_from_usize(value: usize) -> Self::ByteArray {
            (value as Self::Int).to_be_bytes()
        }
    };
}

macro_rules! impl_max_and_byte_array {
    () => {
        const MAX: usize = Self::Int::MAX as usize;
        type ByteArray = [u8; std::mem::size_of::<Self::Int>()];
    };
}

macro_rules! impl_read_raw_unchecked {
    () => {
        #[inline]
        fn read_raw_unchecked(source: &[u8]) -> Self::Int {
            let pointer = source.as_ptr() as *const Self::Int;
            let value: Self::Int = unsafe { pointer.read_unaligned() };

            value.to_be()
        }
    };
}

macro_rules! impl_read {
    () => {
        #[inline]
        fn read(source: &[u8], field_name: &'static str) -> Result<Option<usize>, WireError> {
            let Some(raw_value) = Self::read_raw(source) else {
                return Ok(None);
            };

            let value: usize = raw_value.try_into().map_err(|_| {
                WireError::LengthOverflow(field_name, raw_value as u128, usize::MAX)
            })?;

            Ok(Some(value))
        }
    };
}

macro_rules! impl_read_unchecked {
    () => {
        #[inline]
        fn read_unchecked(source: &[u8], field_name: &'static str) -> Result<usize, WireError> {
            let raw_value = Self::read_raw_unchecked(source);

            let value: usize = raw_value.try_into().map_err(|_| {
                WireError::LengthOverflow(field_name, raw_value as u128, usize::MAX)
            })?;

            Ok(value)
        }
    };
}

macro_rules! impl_wired_int_for {
    ($ty:ty) => {
        impl WiredIntInner for $ty {
            type Int = $ty;

            impl_max_and_byte_array!();

            impl_to_bytes!();
            impl_to_bytes_from_usize!();

            impl_read_raw_unchecked!();

            impl_read!();
            impl_read_unchecked!();
        }
    };
}

impl_wired_int_for!(u8);
impl_wired_int_for!(u16);
impl_wired_int_for!(u32);
impl_wired_int_for!(u64);
impl_wired_int_for!(u128);

pub trait WiredInt {
    type Inner: WiredIntInner;

    const FIELD_NAME: &'static str;
}

pub trait WiredIntInner: Sized {
    type Int: Copy;
    type ByteArray: AsRef<[u8]> + AsMut<[u8]> + Sized;

    const SIZE: usize = std::mem::size_of::<Self::Int>();
    const MAX: usize;

    fn read_raw_unchecked(source: &[u8]) -> Self::Int;
    fn read_raw(source: &[u8]) -> Option<Self::Int> {
        length_check!(source);

        Some(Self::read_raw_unchecked(source))
    }

    fn read_unchecked(source: &[u8], field_name: &'static str) -> Result<usize, WireError>;
    fn read(source: &[u8], field_name: &'static str) -> Result<Option<usize>, WireError>;

    fn to_bytes_from_usize(value: usize) -> Self::ByteArray;
    fn to_bytes(value: Self::Int) -> Self::ByteArray;
}
