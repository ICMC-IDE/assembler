use pest::iterators::Pair;

use super::{arguments::Arguments, Reduce, ReduceError, Statement};
use crate::{asm::Rule, context::Context};

#[derive(Debug)]
pub struct Instruction<'a> {
    pub arguments: Arguments<'a>,
    pub pair: Pair<'a, Rule>,
}

impl<'a> Reduce for Instruction<'a> {
    type Error = ReduceError<'a>;
    type Output = Option<Statement<'a>>;

    fn reduce(self, ctx: &mut Context) -> Result<Self::Output, Self::Error> {
        let mnemonics = ctx.is.get_instruction(self.pair.as_str());

        if let Some(mnemonics) = mnemonics {
            // assumes a instruction cannot have different sizes based on input
            let size = mnemonics[0].length / 16;
            ctx.address += size;

            if self.is_reduced() {
                let result = mnemonics
                    .iter()
                    .map(|mnemonic| {
                        self.arguments
                            .validate_argc(mnemonic.argc())
                            .map_err(|err| err.to_reduce_err(self.pair.clone()))?;

                        self.arguments
                            .iter()
                            .zip(&mnemonic.arguments)
                            .try_fold(mnemonic.value, |acc, (expr, arg)| {
                                expr.validate(ctx, arg).map(|value| value | acc)
                            })
                    })
                    .collect::<Vec<Result<_, ReduceError>>>();

                if let Some(result) = result.iter().find(|x| x.is_ok()) {
                    let bytes = result.as_ref().unwrap().to_be_bytes();

                    let data = bytes[(4 - 2 * size)..]
                        .chunks_exact(2)
                        .map(|bytes| u16::from_be_bytes([bytes[0], bytes[1]]))
                        .collect::<Vec<u16>>();
                    Ok(Some(Statement::Data(data.into_boxed_slice(), None)))
                } else {
                    todo!()
                }
            } else {
                let arguments = self.arguments.reduce(ctx)?;

                Ok(Some(Statement::Instruction(Instruction {
                    arguments,
                    ..self
                })))
            }
        } else {
            Err(ReduceError::UnknownInstruction(self.pair))
        }
    }

    fn is_reduced(&self) -> bool {
        self.arguments.is_reduced()
    }
}
