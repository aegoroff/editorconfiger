use std::collections::{BTreeMap, HashSet};
use std::hash::Hash;

/// Returns only duplicate items iterator
pub fn only_duplicates<T: Eq + Ord>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
    iter.fold(BTreeMap::new(), |mut h, s| {
        *h.entry(s).or_insert(0) += 1;
        h
    })
    .into_iter()
    .filter(|(_, v)| *v > 1)
    .map(|(k, _)| k)
}

/// Returns iterator over unique items from original iterator
pub fn only_unique<T: Eq + Hash + Clone>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
    let mut hs = HashSet::new();
    iter.filter(move |x| {
        // contains call is important so as not to do redundant clone
        if !hs.contains(x) {
            hs.insert(x.clone())
        } else {
            false
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(vec!["a", "b", "b", "a"], vec!["a", "b"])]
    #[case(vec!["a", "b", "b", "a", "a"], vec!["a", "b"])]
    #[case(vec!["a", "b", "b", "a", "a", "a"], vec!["a", "b"])]
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
