mod lexer;

use lexer::Token;
use miette::Result;

/// Named container of properties
#[derive(Default)]
pub struct Section<'a> {
    /// any data between square brackets (i.e. [])
    pub title: &'a str,
    pub properties: Vec<Property<'a>>,
}

/// Property represents name/value pair
pub struct Property<'input> {
    pub name: &'input str,
    pub value: &'input str,
}

/// Parses input str to [`Section`] vector (array).
/// Sections order matches original file sections order.
pub fn parse(content: &str) -> Result<Vec<Section<'_>>> {
    let tokens = lexer::tokenize(content);

    let mut result = vec![];

    for token in tokens {
        match token.map_err(|e| e.with_source_code(content.to_owned()))? {
            Token::Head(h) => {
                let section = Section::<'_> {
                    title: h,
                    ..Default::default()
                };
                result.push(section);
            }
            Token::Pair(k, v) => {
                // root section case i.e. key value pair without any section
                // so we add section with empty title
                if result.is_empty() {
                    result.push(Section::default());
                }
                // because tokens stream has order as in original file
                // it's safe to add key/value pair into the last found section defined
                // by Token::Head or fake root section added before
                if let Some(section) = result.last_mut() {
                    section.properties.push(Property { name: k, value: v });
                }
            }
            // Skip comments so far
            Token::Comment(_) => {}
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_several_sections_len_and_content_as_expected() {
        // Arrange
        let config = r#"
root = true
[*]
a = b
c = d

[*.md]
e = f"#;

        // Act
        let contents = parse(config).expect("Parsing failed");

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
        let config = r#"
[*]
a = b
c = d"#;

        // Act
        let contents = parse(config).expect("Parsing failed");

        // Assert
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[0].properties.len(), 2);
    }
}
