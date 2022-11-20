//! Utilities for numerics.

use std::cmp::Ordering;

/// An `f32` that implements [`Ord`] according to the IEEE 754 totalOrder predicate.
#[derive(Clone, Copy)]
pub struct TotalF32(pub f32);

impl PartialEq for TotalF32 {
    fn eq(&self, other: &Self) -> bool {
        f32::total_cmp(&self.0, &other.0) == Ordering::Equal
    }
}

impl Eq for TotalF32 {}

impl PartialOrd for TotalF32 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for TotalF32 {
    fn cmp(&self, other: &Self) -> Ordering {
        f32::total_cmp(&self.0, &other.0)
    }
}

// TODO remove once libstd feature is stable
pub fn is_sorted<I, T>(what: I) -> bool
where
    I: IntoIterator<Item = T>,
    T: Ord,
{
    let mut iter = what.into_iter();
    let mut last = match iter.next() {
        Some(elem) => elem,
        None => return true, // 0 elements are always sorted
    };

    iter.all(|current| {
        if last > current {
            return false;
        }

        last = current;
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sorted() {
        assert!(is_sorted::<_, u8>([]));
        assert!(is_sorted([0]));
        assert!(is_sorted([0, 1]));
        assert!(is_sorted([0, 1, 2]));
        assert!(!is_sorted([0, 2, 1]));
        assert!(!is_sorted([2, 1]));
        assert!(!is_sorted([1, 0, 2]));
    }
}
