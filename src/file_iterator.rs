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
