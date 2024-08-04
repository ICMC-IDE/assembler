use pest::iterators::Pair;

use super::{Reduce, ReduceError};
use crate::{
    asm::Rule,
    context::{Argument, Context},
};

#[derive(Debug, Clone)]
pub struct Compound<'a> {
    pub pair: Pair<'a, Rule>,
    pub lhs: Box<Expr<'a>>,
    pub rhs: Box<Expr<'a>>,
    pub operator: Operator,
}

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    String {
        pair: Pair<'a, Rule>,
        value: Box<[u16]>,
    },
    Integer {
        pair: Pair<'a, Rule>,
        value: usize,
    },
    LabelRef {
        pair: Pair<'a, Rule>,
        name: &'a str,
    },
    Compound(Compound<'a>),
    Symbol {
        pair: Pair<'a, Rule>,
        name: &'a str,
    },
    Eoi,
}

impl<'a> Expr<'a> {
    pub fn dependencies(&self) -> Vec<&str> {
        match self {
            Self::Compound(Compound { lhs, rhs, .. }) => {
                Iterator::chain(lhs.dependencies().into_iter(), rhs.dependencies()).collect()
            }
            Self::LabelRef { name, .. } | Self::Symbol { name, .. } => [*name].into(),
            _ => [].into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Add,
    Sub,
}

impl<'a> From<Pair<'a, Rule>> for Expr<'a> {
    fn from(pair: Pair<'a, Rule>) -> Self {
        match pair.as_rule() {
            Rule::argument => pair.into_inner().next().unwrap().into(),
            Rule::ident | Rule::word => Self::LabelRef {
                name: pair.as_str(),
                pair,
            },
            Rule::number => Self::Integer {
                value: pair.as_str().parse().unwrap(),
                pair,
            },
            Rule::char => {
                let string = pair.as_str();
                let string = &string[1..(string.len() - 1)];
                let mut chars = string.chars();
                let mut chr = chars.next().unwrap() as usize;

                if chr == '\\' as usize {
                    let next = chars.next().unwrap();

                    chr = match next {
                        '\\' | '\'' => next as usize,
                        _ => string[1..].parse().unwrap(),
                    };
                }

                Self::Integer { value: chr, pair }
            }
            Rule::expr => {
                let mut pairs = pair.clone().into_inner();
                let lhs = pairs.next().unwrap().into();

                if let Some(operator) = pairs.next() {
                    let rhs = pairs.next().unwrap().into();
                    let operator = match operator.as_str() {
                        "+" => Operator::Add,
                        "-" => Operator::Sub,
                        _ => unreachable!(),
                    };

                    Self::Compound(Compound {
                        pair,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        operator,
                    })
                } else {
                    lhs
                }
            }
            Rule::string => {
                let mut buffer = Vec::new();
                let string = pair.as_str();
                let string = &string[1..(string.len() - 1)];
                let mut chars = string.bytes();

                while let Some(chr) = chars.next() {
                    match chr {
                        b'\\' => match chars.next().unwrap() {
                            b'0' => buffer.push(0),
                            next => buffer.push(next as u16),
                        },
                        chr => buffer.push(chr as u16),
                    }
                }

                buffer.push(0);

                Self::String {
                    pair,
                    value: buffer.into_boxed_slice(),
                }
            }
            Rule::EOI => Self::Eoi,
            rule => unreachable!("Unimplemented for {rule:#?}"),
        }
    }
}

impl<'a> Reduce for Expr<'a> {
    type Error = ReduceError<'a>;
    type Output = Self;

    fn reduce(self, ctx: &mut Context) -> Result<Self::Output, Self::Error> {
        match self {
            Self::Compound(Compound {
                pair,
                lhs,
                rhs,
                operator,
            }) => {
                let lhs = lhs.reduce(ctx)?;
                let rhs = rhs.reduce(ctx)?;

                // Reduces expressions to integers
                match (lhs, rhs, operator) {
                    (Self::Integer { value: x, .. }, Self::Integer { value: y, .. }, op) => {
                        Ok(Self::Integer {
                            value: match op {
                                Operator::Add => x.wrapping_add(y),
                                Operator::Sub => x.wrapping_sub(y),
                            },
                            pair,
                        })
                    }
                    (lhs, rhs, operator) => Ok(Self::Compound(Compound {
                        pair,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                        operator,
                    })),
                }
            }
            Self::LabelRef { name, pair } => {
                if ctx.is.get_symbol(name).is_some() {
                    Ok(Self::Symbol { name, pair })
                } else if let Some(Some(address)) = ctx.labels.get(name) {
                    Ok(Self::Integer {
                        value: *address,
                        pair,
                    })
                } else {
                    Ok(Self::LabelRef { pair, name })
                }
            }
            _ => Ok(self),
        }
    }

    fn is_reduced(&self) -> bool {
        match self {
            Self::Integer { .. } | Self::String { .. } | Self::Symbol { .. } => true,
            Self::Compound(Compound { lhs, rhs, .. }) => lhs.is_reduced() && rhs.is_reduced(),
            _ => false,
        }
    }
}

impl<'a> Expr<'a> {
    pub fn pair(&self) -> Pair<'a, Rule> {
        match self {
            Self::Compound(expr) => expr.pair.clone(),
            Self::Integer { pair, .. }
            | Self::LabelRef { pair, .. }
            | Self::Symbol { pair, .. }
            | Self::String { pair, .. } => pair.clone(),
            _ => unimplemented!(),
        }
    }

    pub fn validate(&self, ctx: &'a Context, arg: &'a Argument) -> Result<u32, ReduceError<'a>> {
        match self {
            Self::Symbol { pair, name } => {
                let symbol = &ctx.is.get_symbol(name).unwrap();
                if symbol.tags.contains(&arg.r#type) {
                    Ok(arg.format(symbol.value as u32))
                } else {
                    Err(ReduceError::ExpectedType {
                        argument: pair.clone(),
                        expected: symbol.tags.iter().map(std::ops::Deref::deref).collect(),
                        found: &arg.r#type,
                    })
                }
            }
            Self::Integer { value, .. } => {
                if let Some((kind, _bits)) = arg.r#type.split_once(|c: char| c.is_ascii_digit()) {
                    // let size = bits.parse().unwrap();
                    let value = match kind {
                        "u" | "i" | "ptr" => *value as u16,
                        _ => unimplemented!(),
                    };

                    Ok(arg.format(value as u32))
                } else {
                    unimplemented!()
                }
            }
            Self::String { value, .. } => {
                if let Some((kind, _bits)) = arg.r#type.split_once(|c: char| c.is_ascii_digit()) {
                    // let size = bits.parse().unwrap();
                    let value = match kind {
                        "u" | "i" | "ptr" => value[0],
                        _ => unimplemented!(),
                    };

                    Ok(arg.format(value as u32))
                } else {
                    unimplemented!()
                }
            }
            expr => unimplemented!("Not implemented for expr of type {expr:#?} and {arg:#?}"),
        }
    }
}
