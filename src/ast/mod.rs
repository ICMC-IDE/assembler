mod arguments;
mod expr;
mod instruction;
mod label;
mod r#macro;
mod statement;

use std::{error::Error, fmt::Display};

pub use arguments::*;
pub use expr::*;
pub use instruction::*;
pub use label::*;
use pest::iterators::Pair;
pub use r#macro::*;
pub use statement::*;

use crate::{
    asm::Rule,
    context::{Context, LabelError},
};

#[derive(Debug)]
pub enum ReduceError<'a> {
    UnknownInstruction(Pair<'a, Rule>),
    UnknownIdentifier(Pair<'a, Rule>),
    TypeError,
    UnexpectedArgument {
        instruction: Pair<'a, Rule>,
        arguments: Vec<Pair<'a, Rule>>,
        expected: usize,
        found: usize,
    },
    ExpectedArgument {
        instruction: Pair<'a, Rule>,
        expected: usize,
        found: usize,
    },
    ExpectedType {
        argument: Pair<'a, Rule>,
        expected: Vec<&'a str>,
        found: &'a str,
    },
    LabelRedeclaration {
        label: Pair<'a, Rule>,
    },
}

pub trait Reduce {
    type Output;
    type Error;

    fn reduce(self, ctx: &mut Context) -> Result<Self::Output, Self::Error>;

    fn is_reduced(&self) -> bool {
        false
    }
}

pub trait Dependencies {
    fn dependencies(&self) -> Vec<&str>;
}

impl<'a> ReduceError<'a> {
    pub fn from_label_err(err: LabelError, label: Pair<'a, Rule>) -> Self {
        match err {
            LabelError::Unavailable => Self::LabelRedeclaration { label },
            LabelError::InvalidLabel => {
                todo!()
            }
        }
    }
}

impl<'a> Display for ReduceError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:#?}"))
    }
}

impl<'a> Error for ReduceError<'a> {}
