macro_rules! length_check {
    ($source:expr) => {
        if $source.len() > Self::WIDTH {
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

pub trait LengthPrefix: Sized {
    type Int: Copy;
    type ByteArray: AsRef<[u8]> + AsMut<[u8]> + Sized;

    const WIDTH: usize;
    const MAX: usize;

    fn read(source: &[u8]) -> Option<usize>;
    fn to_bytes_from_usize(value: usize) -> Self::ByteArray;
    fn to_bytes(value: Self::Int) -> Self::ByteArray;
}

impl LengthPrefix for u8 {
    type Int = u8;
    type ByteArray = [u8; 1];

    const WIDTH: usize = 1;
    const MAX: usize = u8::MAX as usize;

    #[inline]
    fn read(source: &[u8]) -> Option<usize> {
        length_check!(source);

        Some(source[0] as usize)
    }

    impl_to_bytes!();
    impl_to_bytes_from_usize!();
}

impl LengthPrefix for u16 {
    type Int = u16;
    type ByteArray = [u8; 2];

    const WIDTH: usize = 2;
    const MAX: usize = u16::MAX as usize;

    #[inline]
    fn read(source: &[u8]) -> Option<usize> {
        length_check!(source);

        Some(u16::from_be_bytes([source[0], source[1]]) as usize)
    }

    impl_to_bytes!();
    impl_to_bytes_from_usize!();
}

impl LengthPrefix for u32 {
    type Int = u32;
    type ByteArray = [u8; 4];

    const WIDTH: usize = 4;
    const MAX: usize = u32::MAX as usize;

    #[inline]
    fn read(source: &[u8]) -> Option<usize> {
        length_check!(source);

        Some(u32::from_be_bytes([source[0], source[1], source[2], source[3]]) as usize)
    }

    impl_to_bytes!();
    impl_to_bytes_from_usize!();
}

impl LengthPrefix for u64 {
    type Int = u64;
    type ByteArray = [u8; 8];

    const WIDTH: usize = 8;
    const MAX: usize = u64::MAX as usize;

    #[inline]
    fn read(source: &[u8]) -> Option<usize> {
        length_check!(source);

        Some(u64::from_be_bytes([
            source[0], source[1], source[2], source[3], source[4], source[5], source[6], source[7],
        ]) as usize)
    }

    impl_to_bytes!();
    impl_to_bytes_from_usize!();
}

impl LengthPrefix for u128 {
    type Int = u128;
    type ByteArray = [u8; 16];

    const WIDTH: usize = 16;
    const MAX: usize = u128::MAX as usize;

    #[inline]
    fn read(source: &[u8]) -> Option<usize> {
        length_check!(source);

        Some(u128::from_be_bytes([
            source[0], source[1], source[2], source[3], source[4], source[5], source[6], source[7],
            source[8], source[9], source[10], source[11], source[12], source[13], source[14],
            source[15],
        ]) as usize)
    }

    impl_to_bytes!();
    impl_to_bytes_from_usize!();
}
