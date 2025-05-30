//! Hilbert curve ID

#[inline]
pub fn xy_to_hilbert(base_z: u8, mut x: u32, mut y: u32) -> u64 {
    (0..base_z).rev().fold(0, |acc, a| {
        let s = 1 << a;
        let rx = s & x;
        let ry = s & y;
        (x, y) = rotate(s, x, y, rx, ry);
        acc + ((((3 * rx) ^ ry) as u64) << a)
    })
}

#[inline]
pub fn hilbert_to_xy(base_z: u8, id: u64) -> (u32, u32) {
    let mut pos = id;
    let (x, y) = (0..base_z).fold((0, 0), |(x, y), a| {
        let s = 1u32 << a;
        let rx = s & (pos as u32 >> 1);
        let ry = s & (pos as u32 ^ rx);
        let (x, y) = rotate(s, x, y, rx, ry);
        pos >>= 1;
        (x + rx, y + ry)
    });
    (x, y)
}

#[inline]
pub fn hilbert_to_zxy(id: u64) -> (u8, u32, u32) {
    let z = (((u64::BITS - (3 * id + 1).leading_zeros()) - 1) / 2) as u8;
    let acc = ((1 << (z * 2)) - 1) / 3;
    let mut pos = id - acc;
    let (x, y) = (0..z).fold((0, 0), |(x, y), a| {
        let s = 1u32 << a;
        let rx = s & (pos as u32 >> 1);
        let ry = s & (pos as u32 ^ rx);
        let (x, y) = rotate(s, x, y, rx, ry);
        pos >>= 1;
        (x + rx, y + ry)
    });
    (z, x, y)
}

#[inline]
pub fn zxy_to_hilbert(z: u8, mut x: u32, mut y: u32) -> u64 {
    let acc = ((1 << (z * 2)) - 1) / 3;
    (0..z).rev().fold(acc, |acc, a| {
        let s = 1 << a;
        let rx = s & x;
        let ry = s & y;
        (x, y) = rotate(s, x, y, rx, ry);
        acc + ((((3 * rx) ^ ry) as u64) << a)
    })
}

#[inline]
const fn rotate(n: u32, mut x: u32, mut y: u32, rx: u32, ry: u32) -> (u32, u32) {
    if ry == 0 {
        if rx != 0 {
            x = (n - 1).wrapping_sub(x);
            y = (n - 1).wrapping_sub(y);
        }
        (x, y) = (y, x)
    }
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_xy() {
        assert_eq!(xy_to_hilbert(0, 0, 0), 0);
        assert_eq!(xy_to_hilbert(1, 0, 0), 0);
        assert_eq!(xy_to_hilbert(1, 0, 1), 1);
        assert_eq!(xy_to_hilbert(1, 1, 1), 2);
        assert_eq!(xy_to_hilbert(1, 1, 0), 3);
        assert_eq!(xy_to_hilbert(2, 0, 2), 4);

        assert_eq!(hilbert_to_xy(1, xy_to_hilbert(1, 0, 0)), (0, 0));
        assert_eq!(hilbert_to_xy(1, xy_to_hilbert(1, 0, 1)), (0, 1));
        assert_eq!(hilbert_to_xy(1, xy_to_hilbert(1, 1, 0)), (1, 0));
        assert_eq!(hilbert_to_xy(2, xy_to_hilbert(2, 0, 2)), (0, 2));
    }

    #[test]
    fn roundtrip_xyz() {
        let fixture = vec![
            // ((x, y, z), expected_tile_id)
            //
            // z = 0
            ((0, 0, 0), 0),
            // z = 1
            ((1, 0, 0), 1),
            ((1, 0, 1), 2),
            ((1, 1, 1), 3),
            ((1, 1, 0), 4),
            // z = 2
            ((2, 0, 1), 8),
            ((2, 1, 1), 7),
            ((2, 2, 0), 19),
            ((2, 3, 3), 15),
            ((2, 3, 2), 16),
            // z= 3
            ((3, 0, 0), 21),
            ((3, 7, 0), 84),
            // z = 4
            ((4, 0, 0), 85),
            ((4, 15, 0), 340),
            // z = 18 (tileId exceeds u32)
            ((18, 1, 1), 22906492247),
            // z = 31
            ((31, 100, 100), 1537228672809139573),
        ];

        for ((z, x, y), expected_tile_id) in fixture {
            let tile_id = zxy_to_hilbert(z, x, y);
            assert_eq!(tile_id, expected_tile_id);
            assert_eq!(hilbert_to_zxy(tile_id), (z, x, y));
        }
    }
}
