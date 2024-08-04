use pest::iterators::Pair;

use super::{Reduce, ReduceError, Statement};
use crate::{asm::Rule, context::Context};

#[derive(Debug, Clone)]
pub struct Label<'a> {
    pub pair: Pair<'a, Rule>,
    pub registered: bool,
}

impl<'a> Reduce for Label<'a> {
    type Error = ReduceError<'a>;
    type Output = Option<Statement<'a>>;

    fn reduce(self, ctx: &mut Context) -> Result<Self::Output, Self::Error> {
        let label = self.pair.as_str();

        match ctx.register_label(label, self.registered) {
            Ok(_) => Ok(None),
            Err(_error) => Err(ReduceError::LabelRedeclaration { label: self.pair }),
        }
    }
}
