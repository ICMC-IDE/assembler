use crate::context::Context;

use super::{Instruction, Label, Macro, Reduce, ReduceError};

#[derive(Debug)]
pub enum Statement<'a> {
    Label(Label<'a>),
    Instruction(Instruction<'a>),
    Macro(Macro<'a>),
    Data(Box<[u16]>, Option<usize>),
}

impl<'a> Reduce for Statement<'a> {
    type Output = Option<Self>;
    type Error = ReduceError<'a>;

    fn reduce(self, ctx: &mut Context) -> Result<Self::Output, Self::Error> {
        match self {
            Self::Instruction(instruction) => instruction.reduce(ctx),
            Self::Label(label) => label.reduce(ctx),
            Self::Macro(r#macro) => r#macro.reduce(ctx),
            Self::Data(data, offset) => {
                if offset.is_none() {
                    ctx.advance(data.len());
                }

                Ok(Some(Self::Data(data, offset)))
            }
        }
    }
}

impl<'a> Statement<'a> {
    pub fn copy(&self, buffer: &mut [u16], index: usize) -> usize {
        match self {
            Self::Data(data, offset) => {
                if let Some(address) = *offset {
                    buffer[address..(address + data.len())].copy_from_slice(data);
                    index
                } else {
                    buffer[index..(index + data.len())].copy_from_slice(data);
                    index + data.len()
                }
            }
            _ => {
                println!("test");
                0
            }
        }
    }
}
