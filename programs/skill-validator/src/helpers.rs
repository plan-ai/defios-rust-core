pub trait Bytes {
    fn leading_bits(&self) -> u32;
}

impl Bytes for [u8; 32] {
    fn leading_bits(&self) -> u32 {
        let mut count = 0;
        for x in self {
            let bits = x.leading_zeros();
            count += bits;
            if bits != 8 {
                break;
            }
        }
        count
    }
}
