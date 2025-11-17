pub trait LengthPrefix: Sized {
    const WIDTH: usize;

    fn read(source: &[u8]) -> Option<usize>;
}

impl LengthPrefix for u8 {
    const WIDTH: usize = 1;

    #[inline]
    fn read(source: &[u8]) -> Option<usize> {
        if source.is_empty() {
            return None;
        }

        Some(source[0] as usize)
    }
}

impl LengthPrefix for u16 {
    const WIDTH: usize = 2;

    #[inline]
    fn read(source: &[u8]) -> Option<usize> {
        if source.len() < 2 {
            return None;
        }

        Some(u16::from_be_bytes([source[0], source[1]]) as usize)
    }
}

impl LengthPrefix for u32 {
    const WIDTH: usize = 4;

    #[inline]
    fn read(source: &[u8]) -> Option<usize> {
        if source.len() < 4 {
            return None;
        }

        Some(u32::from_be_bytes([source[0], source[1], source[2], source[3]]) as usize)
    }
}

impl LengthPrefix for u64 {
    const WIDTH: usize = 8;

    #[inline]
    fn read(source: &[u8]) -> Option<usize> {
        if source.len() < 8 {
            return None;
        }

        Some(u64::from_be_bytes([
            source[0], source[1], source[2], source[3], source[4], source[5], source[6], source[7],
        ]) as usize)
    }
}
