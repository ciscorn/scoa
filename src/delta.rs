//! Delta encoding utilities

#[inline]
pub fn delta_encode<T: core::ops::Sub<Output = T> + Copy + Default>(
    it: impl IntoIterator<Item = T>,
    a: T,
) -> impl Iterator<Item = T> {
    let mut iter = it.into_iter();
    let first = iter.next();
    let mut prev = first.unwrap_or_default();
    std::iter::once(first).flatten().chain(iter.map(move |x| {
        let result = x - prev - a;
        prev = x;
        result
    }))
}

#[inline]
pub fn delta_decode<T: core::ops::Add<Output = T> + Copy + Default>(
    it: impl IntoIterator<Item = T>,
    a: T,
) -> impl Iterator<Item = T> {
    let mut iter = it.into_iter();
    let first = iter.next();
    std::iter::once(first)
        .flatten()
        .chain(iter.scan(first.unwrap_or_default(), move |prev, x| {
            let result = *prev + x + a;
            *prev = result;
            Some(result)
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_encode() {
        let data = vec![11, 14, 18, 23, 29];
        let decoded: Vec<_> = delta_encode(data, 1).collect();
        assert_eq!(decoded, vec![11, 2, 3, 4, 5]);

        let encoded: Vec<_> = delta_encode([].iter().copied(), 1i32).collect();
        assert!(encoded.is_empty());

        let encoded: Vec<_> = delta_encode([3].iter().copied(), 1i32).collect();
        assert_eq!(encoded, vec![3]);
    }

    #[test]
    fn test_delta_decode() {
        let data = vec![11, 2, 3, 4, 5];
        let decoded: Vec<_> = delta_decode(data, 1).collect();
        assert_eq!(decoded, vec![11, 14, 18, 23, 29]);

        let decoded: Vec<_> = delta_decode([].iter().copied(), 1i32).collect();
        assert!(decoded.is_empty());

        let decoded: Vec<_> = delta_decode([3].iter().copied(), 1i32).collect();
        assert_eq!(decoded, vec![3]);
    }
}
