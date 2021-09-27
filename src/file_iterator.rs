use crate::parser;
use ini::{Ini, Properties, SectionIter};

pub struct FileIterator<I: Iterator> {
    inner: I,
}

impl<'a> FileIterator<SectionIter<'a>> {
    pub fn from(ini: &'a Ini) -> Self {
        Self { inner: ini.iter() }
    }
}

impl<'a> Iterator for &'a mut FileIterator<SectionIter<'a>> {
    type Item = (Vec<String>, &'a Properties);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();

        if let Some((sec, prop)) = item {
            let sk = sec.unwrap_or("root");
            let extensions = parser::parse(sk);
            return Some((extensions, prop));
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
        let extensions: Vec<Vec<String>> = it.map(|(extensions, _props)| extensions).collect();

        // Assert
        assert_that!(extensions).has_length(3);
    }
}
