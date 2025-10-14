use anyhow::{anyhow, Result};
use pest::iterators::Pair;
use pest::Parser;
use std::borrow::Cow;
use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Make the Relationship struct wasm compatible with proper getters and setters
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct Relationship {
    from: String,
    to: String,
    rel_type: String,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Relationship {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new(from: String, to: String, rel_type: String) -> Self {
        Relationship { from, to, rel_type }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn from(&self) -> String {
        self.from.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn to(&self) -> String {
        self.to.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn rel_type(&self) -> String {
        self.rel_type.clone()
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(setter))]
    pub fn set_from(&mut self, from: String) {
        self.from = from;
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(setter))]
    pub fn set_to(&mut self, to: String) {
        self.to = to;
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(setter))]
    pub fn set_rel_type(&mut self, rel_type: String) {
        self.rel_type = rel_type;
    }
}

#[derive(pest_derive::Parser)]
#[grammar = "crt.pest"] // put the grammar file at src/crt.pest
struct CRTParser;

// ---------- AST ----------
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub id: u32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    EntityRef(u32),
    Not(Box<Expr>),
    And(Vec<Expr>), // n-ary AND
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Entity(u32),
    Not,
    And,
    LParen,
    RParen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub id: u32,
    pub segments: Vec<Expr>,
}

fn tokenize_expr(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => {
                chars.next();
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            'N' | 'n' => {
                let mut buf = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphabetic() {
                        buf.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if buf.eq_ignore_ascii_case("NOT") {
                    tokens.push(Token::Not);
                } else {
                    return Err(anyhow!("Unexpected identifier '{buf}' in expression"));
                }
            }
            'A' | 'a' => {
                let mut buf = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphabetic() {
                        buf.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if buf.eq_ignore_ascii_case("AND") {
                    tokens.push(Token::And);
                } else {
                    return Err(anyhow!("Unexpected identifier '{buf}' in expression"));
                }
            }
            'E' | 'e' => {
                chars.next();
                let mut digits = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        digits.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if digits.is_empty() {
                    return Err(anyhow!("Expected digits after entity prefix 'E'"));
                }
                let id: u32 = digits
                    .parse()
                    .map_err(|_| anyhow!("Invalid entity id '{digits}'"))?;
                tokens.push(Token::Entity(id));
            }
            _ => {
                return Err(anyhow!("Unexpected character '{}' in expression", ch));
            }
        }
    }

    Ok(tokens)
}

struct ExprParser {
    tokens: Vec<Token>,
    pos: usize,
}

impl ExprParser {
    fn new(tokens: Vec<Token>) -> Self {
        ExprParser { tokens, pos: 0 }
    }

    fn parse(mut self) -> Result<Expr> {
        let expr = self.parse_and()?;
        if self.peek().is_some() {
            return Err(anyhow!("Unexpected tokens at end of expression"));
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut exprs = vec![self.parse_not()?];
        while matches!(self.peek(), Some(Token::And)) {
            self.bump();
            exprs.push(self.parse_not()?);
        }
        if exprs.len() == 1 {
            Ok(exprs.remove(0))
        } else {
            Ok(Expr::And(exprs))
        }
    }

    fn parse_not(&mut self) -> Result<Expr> {
        if matches!(self.peek(), Some(Token::Not)) {
            self.bump();
            Ok(Expr::Not(Box::new(self.parse_not()?)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.peek().cloned() {
            Some(Token::Entity(id)) => {
                self.bump();
                Ok(Expr::EntityRef(id))
            }
            Some(Token::LParen) => {
                self.bump();
                let expr = self.parse_and()?;
                match self.peek() {
                    Some(Token::RParen) => {
                        self.bump();
                        Ok(expr)
                    }
                    _ => Err(anyhow!("Missing closing ')' in expression")),
                }
            }
            Some(Token::RParen) => Err(anyhow!("Unexpected ')' in expression")),
            None => Err(anyhow!("Unexpected end of expression")),
            _ => Err(anyhow!("Unexpected token in expression")),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn bump(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CRT {
    pub entities: BTreeMap<u32, Entity>,
    pub links: BTreeMap<u32, Link>,
}

// ---------- API ----------
pub fn parse_crt(input: &str) -> Result<CRT> {
    let source: Cow<'_, str> = if input.ends_with('\n') {
        Cow::Borrowed(input)
    } else {
        Cow::Owned(format!("{input}\n"))
    };

    let mut pairs =
        CRTParser::parse(Rule::file, source.as_ref()).map_err(|e| anyhow!("Parse error: {e}"))?;
    let file = pairs.next().unwrap();

    let mut entities = BTreeMap::<u32, Entity>::new();
    let mut links = BTreeMap::<u32, Link>::new();

    for section in file.into_inner() {
        match section.as_rule() {
            Rule::entity_line => {
                let (id, text) = parse_entity_line(section)?;
                if entities.insert(id, Entity { id, text }).is_some() {
                    return Err(anyhow!("Duplicate entity E{id}"));
                }
            }
            Rule::link_line => {
                let link = parse_link_line(section)?;
                let l = link.clone();
                if links.insert(l.id, l).is_some() {
                    return Err(anyhow!("Duplicate link L{}", link.id));
                }
            }
            // headings/blanklines/whitespace are already consumed in the grammar
            _ => {}
        }
    }

    // validate that every referenced entity exists
    validate_refs(&entities, &links)?;
    Ok(CRT { entities, links })
}

// ---------- parsers ----------
fn parse_entity_line(p: Pair<Rule>) -> Result<(u32, String)> {
    // entity_line = { ws* "E" ID "." ws* text eol }
    let mut id: Option<u32> = None;
    let mut label: Option<String> = None;

    for part in p.into_inner() {
        match part.as_rule() {
            Rule::ID => id = Some(part.as_str().parse()?),
            Rule::text => label = Some(part.as_str().trim().to_string()),
            _ => {}
        }
    }
    let id = id.ok_or_else(|| anyhow!("Missing entity ID"))?;
    let text = label.unwrap_or_default();
    if text.is_empty() {
        return Err(anyhow!("Entity E{id} has empty text"));
    }
    Ok((id, text))
}

fn parse_link_line(p: Pair<Rule>) -> Result<Link> {
    // link_line = { ws* "L" ID "." ws* expr ws* ARROW ws* expr eol }
    let mut id: Option<u32> = None;
    let mut exprs: Vec<Pair<Rule>> = Vec::new();

    for part in p.into_inner() {
        match part.as_rule() {
            Rule::ID => id = Some(part.as_str().parse()?),
            Rule::expr => exprs.push(part),
            _ => {}
        }
    }
    if exprs.len() < 2 {
        return Err(anyhow!(
            "Link must have at least one source expr and one target expr (found {})",
            exprs.len()
        ));
    }
    let id = id.ok_or_else(|| anyhow!("Missing link ID"))?;
    let mut segments = Vec::with_capacity(exprs.len());
    for expr_pair in exprs {
        segments.push(parse_expr(expr_pair)?);
    }
    Ok(Link { id, segments })
}

fn parse_expr(p: Pair<Rule>) -> Result<Expr> {
    debug_assert_eq!(p.as_rule(), Rule::expr);
    let text = p
        .as_str()
        .split_once("//")
        .map(|(before, _)| before)
        .unwrap_or_else(|| p.as_str())
        .trim();
    if text.is_empty() {
        return Err(anyhow!("Empty expression"));
    }
    let tokens = tokenize_expr(text)?;
    ExprParser::new(tokens).parse()
}

fn validate_refs(entities: &BTreeMap<u32, Entity>, links: &BTreeMap<u32, Link>) -> Result<()> {
    fn collect(expr: &Expr, out: &mut Vec<u32>) {
        match expr {
            Expr::EntityRef(id) => out.push(*id),
            Expr::Not(inner) => collect(inner, out),
            Expr::And(items) => items.iter().for_each(|e| collect(e, out)),
        }
    }
    for link in links.values() {
        let mut ids = Vec::new();
        for expr in &link.segments {
            collect(expr, &mut ids);
        }
        for id in ids {
            if !entities.contains_key(&id) {
                return Err(anyhow!(
                    "Link L{} references undefined entity E{}",
                    link.id,
                    id
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"Entities
E1. First
E2. Second
E3. Third

Links
L1. (E1 AND E2) → E3
"#;

    #[test]
    fn parses_parenthesized_and() {
        parse_crt(SAMPLE).expect("should parse AND expression in parentheses");
    }

    #[test]
    fn parses_simple_link() {
        let pair = "L1. E1 → E2\n";
        let mut parsed =
            CRTParser::parse(Rule::link_line, pair).expect("simple link line should parse");
        assert!(parsed.next().is_some());
    }

    #[test]
    fn parses_fixture_file() {
        let data = include_str!("../CRT.neo");
        parse_crt(data).expect("fixture CRT.neo should parse");
    }

    #[test]
    fn raw_expr_parses() {
        let result = parse_expr(
            CRTParser::parse(Rule::expr, "(E1 AND E2)")
                .expect("expr should parse")
                .next()
                .unwrap(),
        )
        .expect("parsed expression");

        assert!(matches!(result, Expr::And(_)));
    }
}
