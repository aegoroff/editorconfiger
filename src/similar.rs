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
        let mut result: Vec<(&str, &str)> = Vec::new();
        for item in items {
            for mat in self.machine.find_overlapping_iter(*item) {
                if !mat.is_empty() {
                    let found = &item[mat.start()..mat.end()];
                    if *item != found && (*item).ends_with(found) {
                        result.push((*item, found))
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn find_no_similar() {
        // Arrange
        let items = vec!["csharp_space_before_comma", "space_before_semicolon"];
        let sim = Similar::new(&items);

        // Act
        let result = sim.find(&items);

        // Assert
        assert_that(&result).is_empty();
    }

    #[test]
    fn find_has_similar() {
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
    fn find_has_similar_but_as_prefix() {
        // Arrange
        let items = vec!["block_comment_end", "block_comment"];
        let sim = Similar::new(&items);

        // Act
        let result = sim.find(&items);

        // Assert
        assert_that(&result).is_empty();
    }
}
