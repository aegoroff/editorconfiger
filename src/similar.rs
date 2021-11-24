use aho_corasick::{AhoCorasickBuilder, MatchKind};

/// This function finds all pairs where the first item is a &str
/// and the second it's suffix
///
/// # Example
///
/// ```
/// use editorconfiger::similar;
///
/// let items = vec!["ab", "aba", "b"];
/// let result = similar::find_suffix_pairs(&items);
/// assert_eq!(vec![("ab", "b")], result);
/// ```
pub fn find_suffix_pairs<'a>(items: &[&'a str]) -> Vec<(&'a str, &'a str)> {
    let machine = AhoCorasickBuilder::new()
        .match_kind(MatchKind::Standard)
        .ascii_case_insensitive(true)
        .build(items);

    items
        .iter()
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
    use rstest::*;
    use spectral::prelude::*;

    #[rstest]
    #[case(vec!["a_b_c_d", "b_c_e"], vec![])]
    #[case(vec!["b_c", "a_b_c"], vec![("a_b_c", "b_c")])]
    #[case(vec!["aab", "aaab", "b"], vec![ ("aab", "b"), ("aaab", "aab"), ("aaab", "b")])]
    #[case(vec!["a_b_c", "a_b"], vec![])]
    #[trace]
    fn find_suffix_tests(#[case] items: Vec<&str>, #[case] expected: Vec<(&str, &str)>) {
        // Arrange

        // Act
        let actual = find_suffix_pairs(&items);

        // Assert
        assert_that!(actual).is_equal_to(expected);
    }
}
