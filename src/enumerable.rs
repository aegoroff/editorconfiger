use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Returns only duplicate items iterator
pub fn only_duplicates<T: Eq + Hash + Clone>(
    iter: impl Iterator<Item = T>,
) -> impl Iterator<Item = T> {
    let mut counter = HashMap::new();
    iter.filter_map(move |x| {
        let count = if let Some(count) = counter.get(&x) {
            *count
        } else {
            0
        };
        // to avoid redundant clone in case of many duplicates
        if count < 2 {
            *counter.entry(x.clone()).or_insert(0) += 1;
        }
        if count == 1 {
            Some(x)
        } else {
            None
        }
    })
}

/// Returns iterator over unique items from original iterator
pub fn only_unique<T: Eq + Hash + Clone>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
    let mut hs = HashSet::new();
    iter.filter(move |x| {
        // contains call is important so as not to do redundant clone
        if hs.contains(x) {
            false
        } else {
            hs.insert(x.clone())
        }
    })
}

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
        let result: Vec<&str> = only_duplicates(items.into_iter()).collect();

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
        let result: Vec<&str> = only_unique(items.into_iter()).collect();

        // Assert
        assert_eq!(result, expected);
    }
}
