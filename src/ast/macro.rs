use pest::iterators::Pair;

use crate::{asm::Rule, context::Context};

use super::{Expr, Reduce, ReduceError, Statement, arguments::Arguments};

#[derive(Debug)]
pub struct Macro<'a> {
    pub arguments: Arguments<'a>,
    pub pair: Pair<'a, Rule>,
    pub is_valid: bool,
}

impl<'a> Reduce for Macro<'a> {
    type Output = Option<Statement<'a>>;
    type Error = ReduceError<'a>;

    // static addr, data
    // alloc addr, size
    // string data
    // var size
    fn reduce(self, ctx: &mut Context) -> Result<Self::Output, Self::Error> {
        if !self.is_valid {
            let argc = match self.pair.as_str() {
                "string" | "var" => 1,
                "alloc" | "static" => 2,
                _ => unreachable!(),
            };

            // checks for argument count
            self.arguments
                .validate_argc(argc)
                .map_err(|err| err.to_reduce_err(self.pair.clone()))?;
        }

        let mut arguments = self.arguments.reduce(ctx)?;

        // checks if arguments are resolved and their types
        match self.pair.as_str() {
            "string" => match arguments.expr_list.pop().unwrap() {
                Expr::String { value, .. } => {
                    ctx.advance(value.len());
                    Ok(Some(Statement::Data(value, None)))
                }
                expr @ Expr::LabelRef { .. } => {
                    Ok(Some(Statement::Macro(Self {
                        is_valid: true,
                        arguments: Arguments {
                            expr_list: vec![expr],
                        },
                        ..self
                    })))
                }
                _ => Err(ReduceError::TypeError(self.pair)),
            },
            "var" => match arguments.expr_list.pop().unwrap() {
                Expr::Integer { value, .. } => Ok(Some(Statement::Data(
                    {
                        ctx.advance(value);

                        let vec = vec![0; value];
                        vec.into_boxed_slice()
                    },
                    None,
                ))),
                expr @ Expr::LabelRef { .. } => {
                    Ok(Some(Statement::Macro(Self {
                        is_valid: true,
                        arguments: Arguments {
                            expr_list: vec![expr],
                        },
                        ..self
                    })))
                }
                _ => Err(ReduceError::TypeError(self.pair)),
            },
            "alloc" => {
                let size = arguments.expr_list.pop().unwrap();
                let name = arguments.expr_list.pop().unwrap();

                match (size, name) {
                    (
                        Expr::Integer { value, .. },
                        Expr::LabelRef { name, pair },
                    ) => {
                        ctx.allocate(name, Some(value), false).map_err(
                            |err| ReduceError::from_label_err(err, pair),
                        )?;

                        Ok(None)
                    }
                    (
                        size @ Expr::LabelRef { .. },
                        name @ Expr::LabelRef { .. },
                    ) => Ok(Some(Statement::Macro(Self {
                        is_valid: true,
                        arguments: Arguments {
                            expr_list: vec![name, size],
                        },
                        ..self
                    }))),
                    _ => Err(ReduceError::TypeError(self.pair)),
                }
            }
            "static" => {
                match (&arguments.expr_list[1], &arguments.expr_list[0]) {
                    (
                        Expr::Integer { value, .. },
                        Expr::Integer { value: offset, .. },
                    ) => Ok(Some(Statement::Data(
                        Box::new([*value as u16]),
                        Some(*offset),
                    ))),
                    (
                        Expr::String { value, .. },
                        Expr::Integer { value: offset, .. },
                    ) => Ok(Some(Statement::Data(
                        Box::new([value[0]]),
                        Some(*offset),
                    ))),
                    (Expr::LabelRef { .. }, _) | (_, Expr::LabelRef { .. }) => {
                        Ok(Some(Statement::Macro(Self {
                            is_valid: true,
                            arguments,
                            ..self
                        })))
                    }
                    _ => Err(ReduceError::TypeError(self.pair)),
                }
            }
            _ => unreachable!(),
        }
    }
}
