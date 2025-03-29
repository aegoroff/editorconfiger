use super::lexer::Token;

pub trait Visitor {
    fn visit_string(&self, s: &Str);
    fn visit_set(&self, set: &Set);
    fn visit_list(&self, list: &List);
}

pub trait Expr {
    fn visit<V: Visitor>(&self, visitor: V);
}

pub struct Set<'a> {
    items: Vec<&'a str>,
}

impl Set<'_> {
    pub fn new() -> Self {
        Self { items: vec![] }
    }
}

impl Expr for Set<'_> {
    fn visit<V: Visitor>(&self, visitor: V) {
        visitor.visit_set(self);
    }
}

pub struct List<'a> {
    items: Vec<&'a str>,
}

impl List<'_> {
    pub fn new() -> Self {
        Self { items: vec![] }
    }
}

impl Expr for List<'_> {
    fn visit<V: Visitor>(&self, visitor: V) {
        visitor.visit_list(self);
    }
}

pub struct Str<'a> {
    token: Token<'a>,
}

impl<'a> Str<'a> {
    pub fn new(token: Token<'a>) -> Self {
        Self { token }
    }
}

impl Expr for Str<'_> {
    fn visit<V: Visitor>(&self, visitor: V) {
        visitor.visit_string(self);
    }
}
