use aho_corasick::{AhoCorasickBuilder, MatchKind};

/// This function finds all pairs where the first item is some suffix
/// and the second is a &str which has this suffix but not equal it
pub fn find_suffix_pairs<'a>(items: &[&'a str]) -> Vec<(&'a str, &'a str)> {
    let machine = AhoCorasickBuilder::new()
        .match_kind(MatchKind::Standard)
        .ascii_case_insensitive(true)
        .build(items);

    items
        .into_iter()
        .flat_map(|item| {
            machine
                .find_overlapping_iter(*item)
                .map(move |mat| (*item, mat))
        })
        .filter(|(_item, mat)| !mat.is_empty())
        .map(|(item, mat)| (item, &item[mat.start()..mat.end()]))
        .filter(|(item, found)| *item != *found && (*item).ends_with(*found))
        .map(|(item, found)| (item, found))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn find_suffix_pairs_no_similar_not_found() {
        // Arrange
        let items = vec!["csharp_space_before_comma", "space_before_semicolon"];

        // Act
        let result = find_suffix_pairs(&items);

        // Assert
        assert_that(&result).is_empty();
    }

    #[test]
    fn find_suffix_pairs_different_prefix_found() {
        // Arrange
        let items = vec![
            "csharp_space_before_comma",
            "resharper_csharp_space_before_comma",
        ];

        // Act
        let result = find_suffix_pairs(&items);

        // Assert
        assert_that(&result).has_length(1);
    }

    #[test]
    fn find_suffix_pairs_different_suffix_not_found() {
        // Arrange
        let items = vec!["block_comment_end", "block_comment"];

        // Act
        let result = find_suffix_pairs(&items);

        // Assert
        assert_that(&result).is_empty();
    }
}
