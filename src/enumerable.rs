use std::collections::HashSet;
use std::hash::Hash;

/// Returns only duplicate items iterator
pub fn only_duplicates<T: Eq + Hash + Clone>(
    iter: impl Iterator<Item = T>,
) -> impl Iterator<Item = T> {
    let mut hs = HashSet::new();
    iter.filter(move |x| {
        // contains call is important so as not to do redundant clone
        if !hs.contains(x) {
            !hs.insert(x.clone())
        } else {
            true
        }
    })
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
    use rstest::*;
    use spectral::prelude::*;

    #[rstest]
    #[case(vec!["a", "b", "b", "a"], vec!["b", "a"])]
    #[case(vec!["a", "b", "b"], vec!["b"])]
    #[case(vec!["a", "b"], vec![])]
    #[case(vec![], vec![])]
    #[trace]
    fn only_duplicates_tests(#[case] items: Vec<&str>, #[case] expected: Vec<&str>) {
        // Arrange

        // Act
        let result: Vec<&str> = only_duplicates(items.into_iter()).collect();

        // Assert
        assert_that!(result).is_equal_to(expected);
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
        assert_that!(result).is_equal_to(expected);
    }
}
