use crate::editorconfig_parser::EditorConfigLine;
use crate::{editorconfig_parser, parser, Property};

#[derive(Default)]
pub struct Section<'a> {
    pub title: &'a str,
    pub extensions: Vec<String>,
    pub properties: Vec<Property<'a>>,
}

pub fn parse_file_content<'a>(content: &'a str) -> Vec<Section<'a>> {
    let parsed = editorconfig_parser::parse_editorconfig(content);

    parsed
        .into_iter()
        .fold(Vec::<Section<'a>>::new(), |mut acc, line| {
            match line {
                EditorConfigLine::Head(h) => {
                    let mut section = Section::default();
                    section.title = h;
                    section.extensions = parser::parse(section.title);
                    acc.push(section)
                }
                EditorConfigLine::Pair(k, v) => {
                    if acc.is_empty() {
                        let mut section = Section::default();
                        section.title = "root";
                        section.extensions = parser::parse(section.title);
                        acc.push(section)
                    }
                    let section = acc.last_mut().unwrap();
                    let property = Property {
                        name: k,
                        value: v,
                        section: section.title,
                    };
                    section.properties.push(property);
                }
                EditorConfigLine::Comment(_) => {}
            }

            acc
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn map_several_sections_len_and_content_as_expected() {
        // Arrange
        let config = r###"
root = true
[*]
a = b
c = d

[*.md]
e = f"###;

        // Act
        let contents = parse_file_content(config);

        // Assert
        assert_that!(contents).has_length(3);
        assert_that!(contents.iter().map(|x| x.title).collect::<Vec<&str>>())
            .is_equal_to(vec!["root", "*", "*.md"]);
    }

    #[test]
    fn map_test_properties_len_as_expected() {
        // Arrange
        let config = r###"
[*]
a = b
c = d"###;

        // Act
        let contents = parse_file_content(config);

        // Assert
        assert_that!(contents).has_length(1);
        assert_that!(contents[0].properties).has_length(2);
    }
}
