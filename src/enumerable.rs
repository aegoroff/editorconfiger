use std::collections::HashMap;
use std::hash::Hash;

/// Returns only duplicate items iterator
pub fn only_duplicates<T: Eq + Hash>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
    iter.fold(HashMap::new(), |mut h, s| {
        *h.entry(s).or_insert(0) += 1;
        h
    })
    .into_iter()
    .filter(|(_, v)| *v > 1)
    .map(|(k, _)| k)
}

/// Returns iterator over unique items from original iterator
pub fn only_unique<T: Eq + Hash>(iter: impl Iterator<Item = T>) -> impl Iterator<Item = T> {
    iter.fold(HashMap::new(), |mut h, s| {
        *h.entry(s).or_insert(0) += 1;
        h
    })
    .into_iter()
    .map(|(k, _)| k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn only_duplicates_has_duplicates_not_empty() {
        // Arrange
        let items = vec!["a", "b", "b"];

        // Act
        let result: Vec<&str> = only_duplicates(items.into_iter()).collect();

        // Assert
        assert_that!(result).has_length(1);
    }

    #[test]
    fn only_duplicates_no_duplicates_empty() {
        // Arrange
        let items = vec!["a", "b"];

        // Act
        let result: Vec<&str> = only_duplicates(items.into_iter()).collect();

        // Assert
        assert_that!(result).is_empty();
    }

    #[test]
    fn only_duplicates_empty_iter_empty() {
        // Arrange
        let items: Vec<&str> = vec![];

        // Act
        let result: Vec<&str> = only_duplicates(items.into_iter()).collect();

        // Assert
        assert_that!(result).is_empty();
    }

    #[test]
    fn only_unique_has_duplicates_len_as_expected() {
        // Arrange
        let items = vec!["a", "b", "b"];

        // Act
        let result: Vec<&str> = only_unique(items.into_iter()).collect();

        // Assert
        assert_that!(result).has_length(2);
    }

    #[test]
    fn only_unique_no_duplicates_len_as_expected() {
        // Arrange
        let items = vec!["a", "b"];

        // Act
        let result: Vec<&str> = only_unique(items.into_iter()).collect();

        // Assert
        assert_that!(result).has_length(2);
    }

    #[test]
    fn only_unique_empty_iter_empty_empty_result() {
        // Arrange
        let items: Vec<&str> = vec![];

        // Act
        let result: Vec<&str> = only_unique(items.into_iter()).collect();

        // Assert
        assert_that!(result).is_empty();
    }
}
