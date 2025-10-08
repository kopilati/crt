use anyhow::{anyhow, Result};
use pest::iterators::Pair;
use pest::Parser;
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
pub struct Link {
    pub id: u32,
    pub from: Expr,
    pub to: Expr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CRT {
    pub entities: BTreeMap<u32, Entity>,
    pub links: BTreeMap<u32, Link>,
}

// ---------- API ----------
pub fn parse_crt(input: &str) -> Result<CRT> {
    let mut pairs = CRTParser::parse(Rule::file, input).map_err(|e| anyhow!("Parse error: {e}"))?;
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
    if exprs.len() != 2 {
        return Err(anyhow!(
            "Link must have exactly one source expr and one target expr (found {})",
            exprs.len()
        ));
    }
    let id = id.ok_or_else(|| anyhow!("Missing link ID"))?;
    let from = parse_expr(exprs[0].clone())?;
    let to = parse_expr(exprs[1].clone())?;
    Ok(Link { id, from, to })
}

fn parse_expr(p: Pair<Rule>) -> Result<Expr> {
    debug_assert_eq!(p.as_rule(), Rule::expr);
    // expr -> and_expr
    let inner = p.into_inner().next().unwrap();
    parse_and_expr(inner)
}

fn parse_and_expr(p: Pair<Rule>) -> Result<Expr> {
    // and_expr = not_expr ( "AND" not_expr )*
    let mut exprs = Vec::new();
    for part in p.into_inner() {
        if part.as_rule() == Rule::not_expr {
            exprs.push(parse_not_expr(part)?);
        }
    }
    Ok(if exprs.len() == 1 {
        exprs.remove(0)
    } else {
        Expr::And(exprs)
    })
}

fn parse_not_expr(p: Pair<Rule>) -> Result<Expr> {
    // not_expr = ("NOT" ws+)? primary
    let mut negate = false;
    let mut prim: Option<Pair<Rule>> = None;

    for part in p.clone().into_inner() {
        match part.as_rule() {
            Rule::primary => prim = Some(part),
            _ => {
                if part.as_str().eq_ignore_ascii_case("NOT") {
                    negate = true;
                }
            }
        }
    }

    let mut e = parse_primary(prim.ok_or_else(|| anyhow!("Missing primary in NOT expr"))?)?;
    if negate {
        e = Expr::Not(Box::new(e));
    }
    Ok(e)
}

fn parse_primary(p: Pair<Rule>) -> Result<Expr> {
    match p.as_rule() {
        Rule::entity_ref => parse_entity_ref(p),
        Rule::primary => {
            // "(" expr ")"
            let mut inner = p.into_inner();
            let first = inner.next().ok_or_else(|| anyhow!("Empty primary"))?;
            match first.as_rule() {
                Rule::entity_ref => parse_entity_ref(first),
                Rule::expr => parse_expr(first),
                _ => Err(anyhow!("Invalid primary content")),
            }
        }
        Rule::expr => parse_expr(p),
        _ => Err(anyhow!("Invalid primary")),
    }
}

fn parse_entity_ref(p: Pair<Rule>) -> Result<Expr> {
    let id_pair = p.into_inner().find(|pp| pp.as_rule() == Rule::ID)
        .ok_or_else(|| anyhow!("Missing ID in entity_ref"))?;
    let id: u32 = id_pair.as_str().parse()?;
    Ok(Expr::EntityRef(id))
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
        collect(&link.from, &mut ids);
        collect(&link.to, &mut ids);
        for id in ids {
            if !entities.contains_key(&id) {
                return Err(anyhow!("Link L{} references undefined entity E{}", link.id, id));
            }
        }
    }
    Ok(())
}