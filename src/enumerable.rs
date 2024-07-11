use std::collections::{HashMap, HashSet};
use std::hash::Hash;

pub trait IteratorExt: Iterator {
    /// Filters out duplicate items from the original iterator, returning only unique items.
    fn unique(self) -> UniqueIterator<Self>
    where
        Self: Sized,
        Self::Item: Eq + Hash + Clone,
    {
        UniqueIterator {
            iter: self,
            seen: HashSet::new(),
        }
    }

    /// Filters out items from the original iterator, returning only those that appear more than once.
    fn only_duplicates(self) -> OnlyDuplicatesIterator<Self>
    where
        Self: Sized,
        Self::Item: Eq + Hash + Clone,
    {
        OnlyDuplicatesIterator {
            iter: self,
            counter: HashMap::new(),
        }
    }
}

/// Returns an iterator that contains only the unique elements from `self`.
///
/// This iterator is a drop-in replacement for `self` until it has seen each item in
/// `self`. After that, no more items will be yielded. The order of items is preserved.
///
pub struct UniqueIterator<I>
where
    I: Iterator,
    I::Item: Eq + Hash + Clone,
{
    iter: I,
    seen: HashSet<I::Item>,
}

/// Returns an iterator that yields only the items from `self` that appear more than once in the original iteration.
pub struct OnlyDuplicatesIterator<I>
where
    I: Iterator,
    I::Item: Eq + Hash + Clone,
{
    iter: I,
    counter: HashMap<I::Item, i32>,
}

impl<I> Iterator for UniqueIterator<I>
where
    I: Iterator,
    I::Item: Eq + Hash + Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let x = self.iter.next()?;
            if !self.seen.contains(&x) {
                self.seen.insert(x.clone());
                return Some(x);
            }
        }
    }
}

impl<I> Iterator for OnlyDuplicatesIterator<I>
where
    I: Iterator,
    I::Item: Eq + Hash + Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let x = self.iter.next()?;
            let count = if let Some(count) = self.counter.get(&x) {
                *count
            } else {
                0
            };
            // to avoid redundant clone in case of many duplicates
            if count < 2 {
                *self.counter.entry(x.clone()).or_insert(0) += 1;
            }
            if count == 1 {
                return Some(x);
            }
        }
    }
}

impl<I: Iterator> IteratorExt for I {}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(vec!["a", "b", "b", "a"], vec!["b", "a"])]
    #[case(vec!["a", "b", "b", "a", "a"], vec!["b", "a"])]
    #[case(vec!["a", "b", "b", "a", "a", "a"], vec!["b", "a"])]
    #[case(vec!["a", "b", "b"], vec!["b"])]
    #[case(vec!["a", "b"], vec![])]
    #[case(vec![], vec![])]
    #[trace]
    fn only_duplicates_tests(#[case] items: Vec<&str>, #[case] expected: Vec<&str>) {
        // Arrange

        // Act
        let result: Vec<&str> = items.into_iter().only_duplicates().collect();

        // Assert
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(vec!["a", "a", "b", "b", "c"], vec!["a", "b", "c"])]
    #[case(vec!["a", "b", "b"], vec!["a", "b"])]
    #[case(vec!["a", "b"], vec!["a", "b"])]
    #[case(vec![], vec![])]
    #[trace]
    fn only_unique_tests(#[case] items: Vec<&str>, #[case] expected: Vec<&str>) {
        // Arrange

        // Act
        let result: Vec<&str> = items.into_iter().unique().collect();

        // Assert
        assert_eq!(result, expected);
    }
}
