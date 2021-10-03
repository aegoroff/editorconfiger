use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};

pub struct Similar {
    machine: AhoCorasick,
}

impl Similar {
    pub fn new(items: &[&str]) -> Self {
        let machine = AhoCorasickBuilder::new()
            .match_kind(MatchKind::Standard)
            .ascii_case_insensitive(true)
            .build(items);
        Self { machine }
    }

    pub fn find<'a>(&self, items: &[&'a str]) -> Vec<(&'a str, &'a str)> {
        items
            .into_iter()
            .flat_map(|item| {
                self.machine
                    .find_overlapping_iter(*item)
                    .map(move |mat| (*item, mat))
            })
            .filter(|(_item, mat)| !mat.is_empty())
            .map(|(item, mat)| (item, &item[mat.start()..mat.end()]))
            .filter(|(item, found)| *item != *found && (*item).ends_with(*found))
            .map(|(item, found)| (item, found))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn find_no_similar_not_found() {
        // Arrange
        let items = vec!["csharp_space_before_comma", "space_before_semicolon"];
        let sim = Similar::new(&items);

        // Act
        let result = sim.find(&items);

        // Assert
        assert_that(&result).is_empty();
    }

    #[test]
    fn find_different_prefix_found() {
        // Arrange
        let items = vec![
            "csharp_space_before_comma",
            "resharper_csharp_space_before_comma",
        ];
        let sim = Similar::new(&items);

        // Act
        let result = sim.find(&items);

        // Assert
        assert_that(&result).has_length(1);
    }

    #[test]
    fn find_different_suffix_not_found() {
        // Arrange
        let items = vec!["block_comment_end", "block_comment"];
        let sim = Similar::new(&items);

        // Act
        let result = sim.find(&items);

        // Assert
        assert_that(&result).is_empty();
    }
}
