use std::cmp::Ordering;

use pest::iterators::Pair;

use crate::{asm::Rule, context::Context};

use super::{Expr, Reduce, ReduceError};

#[derive(Debug)]
pub struct Arguments<'a> {
    pub expr_list: Vec<Expr<'a>>,
}

#[derive(Debug)]
pub enum ArgumentError<'a> {
    Expected {
        expected: usize,
        found: usize,
    },
    Unexpected {
        expected: usize,
        found: usize,
        arguments: Vec<Pair<'a, Rule>>,
    },
}

impl<'a> Reduce for Arguments<'a> {
    type Error = ReduceError<'a>;
    type Output = Self;

    fn reduce(self, ctx: &mut Context) -> Result<Self::Output, Self::Error> {
        Ok(Self {
            expr_list: self
                .expr_list
                .into_iter()
                .map(|arg| arg.reduce(ctx))
                .collect::<Result<_, _>>()?,
        })
    }

    fn is_reduced(&self) -> bool {
        self.expr_list.iter().all(|expr| expr.is_reduced())
    }
}

impl<'a> Arguments<'a> {
    pub fn iter(&self) -> impl Iterator<Item = &Expr<'a>> {
        self.expr_list.iter()
    }

    pub fn validate_argc(&self, n: usize) -> Result<(), ArgumentError<'a>> {
        let m = self.expr_list.len();

        match m.cmp(&n) {
            Ordering::Equal => Ok(()),
            Ordering::Less => Err(ArgumentError::Expected {
                expected: n,
                found: m,
            }),
            Ordering::Greater => Err(ArgumentError::Unexpected {
                expected: n,
                found: m,
                arguments: self.expr_list[n..].iter().map(|arg| arg.pair()).collect(), // FIXME: this creates a copy of the array
            }),
        }
    }
}

impl<'a, T> From<T> for Arguments<'a>
where
    T: Into<Vec<Expr<'a>>>,
{
    fn from(value: T) -> Self {
        Self {
            expr_list: value.into(),
        }
    }
}

impl<'a> ArgumentError<'a> {
    pub fn to_reduce_err(self, pair: Pair<'a, Rule>) -> ReduceError<'a> {
        match self {
            Self::Expected { expected, found } => ReduceError::ExpectedArgument {
                instruction: pair,
                expected,
                found,
            },
            Self::Unexpected {
                expected,
                found,
                arguments,
            } => ReduceError::UnexpectedArgument {
                instruction: pair,
                arguments,
                expected,
                found,
            },
        }
    }
}
