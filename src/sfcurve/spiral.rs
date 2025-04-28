//! Spiral curve ID

#[inline]
pub fn xy_to_spiral(x: i16, y: i16) -> u32 {
    let x = x as i32;
    let y = y as i32;
    let d = x.abs() + y.abs();
    let start = if d == 0 { 0 } else { d * (d - 1) * 2 + 1 };

    (if x > 0 && y >= 0 {
        start + d - x
    } else if x <= 0 && y > 0 {
        start + 2 * d - y
    } else if x < 0 && y <= 0 {
        start + 3 * d + x
    } else {
        start + 4 * d + y
    }) as u32
}

#[inline]
pub fn spiral_to_xy(x: u32) -> (i16, i16) {
    if x == 0 {
        return (0, 0);
    }
    let d = {
        let mut d = 0;
        loop {
            let next_start = (d + 1) * d * 2 + 1;
            if x < next_start {
                break;
            }
            d += 1;
        }
        d
    };
    let a = x - (d * (d - 1) * 2 + 1);
    if a < d {
        ((d - a) as i16, a as i16)
    } else if a < 2 * d {
        (-((a - d) as i16), (2 * d - a) as i16)
    } else if a < 3 * d {
        (-((3 * d - a) as i16), -((a - 2 * d) as i16))
    } else {
        ((a - 3 * d) as i16, -((4 * d - a) as i16))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spiral_roundtrip() {
        assert_eq!(xy_to_spiral(0, 0), 0);
        assert_eq!(xy_to_spiral(1, 0), 1);
        assert_eq!(xy_to_spiral(0, 1), 2);
        assert_eq!(xy_to_spiral(-1, 0), 3);
        assert_eq!(xy_to_spiral(0, -1), 4);
        assert_eq!(xy_to_spiral(2, 0), 5);
        assert_eq!(xy_to_spiral(1, 1), 6);
        assert_eq!(xy_to_spiral(0, 2), 7);
        assert_eq!(xy_to_spiral(-1, 1), 8);
        assert_eq!(xy_to_spiral(-2, 0), 9);
        assert_eq!(xy_to_spiral(-1, -1), 10);
        assert_eq!(xy_to_spiral(0, -2), 11);
        assert_eq!(xy_to_spiral(1, -1), 12);
        assert_eq!(xy_to_spiral(3, 0), 13);
        assert_eq!(xy_to_spiral(2, 1), 14);

        assert_eq!(spiral_to_xy(xy_to_spiral(0, 0)), (0, 0));
        assert_eq!(spiral_to_xy(xy_to_spiral(1, 0)), (1, 0));
        assert_eq!(spiral_to_xy(xy_to_spiral(0, 1)), (0, 1));
        assert_eq!(spiral_to_xy(xy_to_spiral(-1, 0)), (-1, 0));
        assert_eq!(spiral_to_xy(xy_to_spiral(0, -1)), (0, -1));
        assert_eq!(spiral_to_xy(xy_to_spiral(2, 0)), (2, 0));
        assert_eq!(spiral_to_xy(xy_to_spiral(1, 1)), (1, 1));
        assert_eq!(spiral_to_xy(xy_to_spiral(0, 2)), (0, 2));
        assert_eq!(spiral_to_xy(xy_to_spiral(-1, 1)), (-1, 1));
        assert_eq!(spiral_to_xy(xy_to_spiral(-2, 0)), (-2, 0));
        assert_eq!(spiral_to_xy(xy_to_spiral(-1, -1)), (-1, -1));
        assert_eq!(spiral_to_xy(xy_to_spiral(0, -2)), (0, -2));
        assert_eq!(spiral_to_xy(xy_to_spiral(1, -1)), (1, -1));
        assert_eq!(spiral_to_xy(xy_to_spiral(3, 0)), (3, 0));
        assert_eq!(spiral_to_xy(xy_to_spiral(2, 1)), (2, 1));
    }
}
