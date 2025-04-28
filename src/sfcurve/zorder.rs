#[inline]
pub fn xy_to_zorder(x: u32, y: u32) -> u64 {
    let x = x as u64;
    let y = y as u64;

    let x = (x | (x << 16)) & 0x0000FFFF0000FFFF;
    let x = (x | (x << 8)) & 0x00FF00FF00FF00FF;
    let x = (x | (x << 4)) & 0x0F0F0F0F0F0F0F0F;
    let x = (x | (x << 2)) & 0x3333333333333333;
    let x = (x | (x << 1)) & 0x5555555555555555;

    let y = (y | (y << 16)) & 0x0000FFFF0000FFFF;
    let y = (y | (y << 8)) & 0x00FF00FF00FF00FF;
    let y = (y | (y << 4)) & 0x0F0F0F0F0F0F0F0F;
    let y = (y | (y << 2)) & 0x3333333333333333;
    let y = (y | (y << 1)) & 0x5555555555555555;

    x + (y << 1)
}

#[inline]
pub fn zorder_to_xy(id: u64) -> (u32, u32) {
    let x = id;
    let y = id >> 1;

    let x = x & 0x5555555555555555;
    let x = (x | (x >> 1)) & 0x3333333333333333;
    let x = (x | (x >> 2)) & 0x0F0F0F0F0F0F0F0F;
    let x = (x | (x >> 4)) & 0x00FF00FF00FF00FF;
    let x = (x | (x >> 8)) & 0x0000FFFF0000FFFF;
    let x = (x | (x >> 16)) & 0x00000000FFFFFFFF;

    let y = y & 0x5555555555555555;
    let y = (y | (y >> 1)) & 0x3333333333333333;
    let y = (y | (y >> 2)) & 0x0F0F0F0F0F0F0F0F;
    let y = (y | (y >> 4)) & 0x00FF00FF00FF00FF;
    let y = (y | (y >> 8)) & 0x0000FFFF0000FFFF;
    let y = (y | (y >> 16)) & 0x00000000FFFFFFFF;

    (x as u32, y as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zorder_encoding() {
        assert_eq!(xy_to_zorder(7313, 3007), 31116203);
        assert_eq!(zorder_to_xy(31116203), (7313, 3007));

        assert_eq!(xy_to_zorder(0, 0), 0);
        assert_eq!(zorder_to_xy(0), (0, 0));

        assert_eq!(xy_to_zorder(0xFFFFFFFF, 0xFFFFFFFF), 0xFFFFFFFFFFFFFFFF);
        assert_eq!(zorder_to_xy(0xFFFFFFFFFFFFFFFF), (0xFFFFFFFF, 0xFFFFFFFF));
    }
}
