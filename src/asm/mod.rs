use pest::Parser;

use crate::ast::{Arguments, Instruction, Label, Macro, Statement};

#[derive(pest_derive::Parser)]
#[grammar = "./asm/syntax.pest"]
pub struct AsmParser;

pub fn parse_line(input: &str) -> Option<impl Iterator<Item = Statement<'_>>> {
    let mut pairs = AsmParser::parse(Rule::line, input).ok()?;

    Some(
        pairs
            .next()
            .unwrap()
            .into_inner()
            .filter(|pair| pair.as_rule() != Rule::EOI)
            .map(|pair| match pair.as_rule() {
                Rule::label => Statement::Label(Label {
                    pair,
                    registered: false,
                }),
                Rule::instruction => {
                    let mut pairs = pair.into_inner();
                    let name = pairs.next().unwrap();
                    let is_macro = matches!(name.as_str(), "string" | "var" | "static" | "alloc");
                    let arguments =
                        Arguments::from(pairs.map(|pair| pair.into()).collect::<Vec<_>>());

                    if is_macro {
                        Statement::Macro(Macro {
                            pair: name,
                            arguments,
                            is_valid: false,
                        })
                    } else {
                        Statement::Instruction(Instruction {
                            pair: name,
                            arguments,
                        })
                    }
                }
                _ => unimplemented!(),
            }),
    )
}
