use crate::{parser, Property};
use ini::{Ini, SectionIter};

pub struct Section<'a> {
    pub title: &'a str,
    pub extensions: Vec<String>,
    pub properties: Vec<Property<'a>>,
}

pub struct FileIterator<I: Iterator> {
    inner: I,
}

impl<'a> FileIterator<SectionIter<'a>> {
    pub fn from(ini: &'a Ini) -> Self {
        Self { inner: ini.iter() }
    }
}

impl<'a> Iterator for &'a mut FileIterator<SectionIter<'a>> {
    type Item = Section<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();

        if let Some((sec, prop)) = item {
            let section = sec.unwrap_or("root");
            let extensions = parser::parse(section);

            let properties: Vec<Property> = prop
                .iter()
                .map(|(k, v)| Property {
                    name: k,
                    value: v,
                    section,
                })
                .collect();

            return Some(Section {
                title: section,
                extensions,
                properties,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn map_several_sections() {
        // Arrange
        let config = r###"
root = true
[*]
a = b
c = d

[*.md]
e = f"###;
        let conf = Ini::load_from_str(config).unwrap();
        let it = &mut FileIterator::from(&conf);

        // Act
        let contents: Vec<Section<'_>> = it.map(|content| content).collect();

        // Assert
        assert_that!(contents).has_length(3);
        assert_that!(contents.iter().map(|x| x.title).collect::<Vec<&str>>())
            .is_equal_to(vec!["root", "*", "*.md"]);
    }

    #[test]
    fn map_test_properties() {
        // Arrange
        let config = r###"
[*]
a = b
c = d"###;
        let conf = Ini::load_from_str(config).unwrap();
        let it = &mut FileIterator::from(&conf);

        // Act
        let props: Vec<Vec<Property<'_>>> = it.map(|x| x.properties).collect();

        // Assert
        assert_that!(props).has_length(1);
        assert_that!(props[0]).has_length(2);
    }
}
