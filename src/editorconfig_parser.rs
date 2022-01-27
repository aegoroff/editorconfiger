use crate::editorconfig_lexer::Token;
use crate::{editorconfig_lexer, glob, Property};

#[derive(Default)]
pub struct Section<'a> {
    pub title: &'a str,
    pub extensions: Vec<String>,
    pub properties: Vec<Property<'a>>,
}

pub fn parse<'a>(content: &'a str) -> Vec<Section<'a>> {
    let tokens = editorconfig_lexer::tokenize(content);

    tokens.fold(Vec::<Section<'a>>::new(), |mut result, token| {
        match token {
            Token::Head(h) => {
                let section = Section::<'a> {
                    title: h,
                    extensions: glob::parse(h),
                    ..Default::default()
                };
                result.push(section)
            }
            Token::Pair(k, v) => {
                // root section case i.e. key value pair without any section
                if result.is_empty() {
                    let section = Section::<'a> {
                        extensions: glob::parse("*"),
                        ..Default::default()
                    };
                    result.push(section)
                }
                // because tokens stream has order as in original file
                // it's safe to add key/value pair into the last found section defined
                // by Token::Head or fake root section added before
                if let Some(section) = result.last_mut() {
                    let property = Property {
                        name: k,
                        value: v,
                        section: section.title,
                    };
                    section.properties.push(property);
                }
            }
            // Skip comments so far
            Token::Comment(_) => {}
        }

        result
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let contents = parse(config);

        // Assert
        assert_eq!(contents.len(), 3);
        assert_eq!(
            contents.iter().map(|x| x.title).collect::<Vec<&str>>(),
            vec!["", "*", "*.md"]
        );
    }

    #[test]
    fn map_test_properties_len_as_expected() {
        // Arrange
        let config = r###"
[*]
a = b
c = d"###;

        // Act
        let contents = parse(config);

        // Assert
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].properties.len(), 2);
    }
}
