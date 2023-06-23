pub fn leading_bits(arr: &[u8; 32]) -> u32 {
    let mut count = 0;
    for x in arr {
        let bits = x.leading_zeros();
        count += bits;
        if bits != 8 {
            break;
        }
    }
    count
}
