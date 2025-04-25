use aho_corasick::{AhoCorasickBuilder, MatchKind};

/// This function finds all pairs where the second item is the suffix of the first one
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
#[must_use]
pub fn find_suffix_pairs<'a>(items: &[&'a str]) -> Vec<(&'a str, &'a str)> {
    let machine = AhoCorasickBuilder::new()
        .match_kind(MatchKind::Standard)
        .ascii_case_insensitive(true)
        .build(items);

    if let Ok(aho) = machine {
        items
            .iter()
            .flat_map(|item| {
                aho.find_overlapping_iter(*item)
                    .map(move |mat| (*item, mat))
            })
            .filter(|(_item, mat)| !mat.is_empty())
            .map(|(item, mat)| (item, &item[mat.start()..mat.end()]))
            .filter(|(item, found)| *item != *found && (*item).ends_with(*found))
            .collect()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(vec!["a_b_c_d", "b_c_e"], vec![]; "case 1")]
    #[test_case(vec!["b_c", "a_b_c"], vec![("a_b_c", "b_c")]; "case 2")]
    #[test_case(vec!["aab", "aaab", "b"], vec![ ("aab", "b"), ("aaab", "aab"), ("aaab", "b")]; "case 3")]
    #[test_case(vec!["a_b_c", "a_b"], vec![]; "case 4")]
    fn find_suffix_tests(items: Vec<&str>, expected: Vec<(&str, &str)>) {
        // Arrange

        // Act
        let actual = find_suffix_pairs(&items);

        // Assert
        assert_eq!(actual, expected);
    }
}
