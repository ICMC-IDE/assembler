use std::{collections::HashMap, fmt::Write, path::Path, str::FromStr};

use asm::parse_line;
use ast::{Reduce, ReduceError};
use context::Context;

pub mod asm;
pub mod ast;
pub mod cis;
pub mod context;

#[derive(Debug)]
pub struct AsmError<'a> {
    pub err: ReduceError<'a>,
    pub line: u32,
    pub expected: Option<usize>,
    pub found: Option<usize>,
}

impl<'a> std::fmt::Display for AsmError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (msg, what) = match &self.err {
            ReduceError::UnknownInstruction(instruction) => {
                ("Unknown Instruction", instruction.as_str().to_owned())
            }
            ReduceError::UnknownIdentifier(instruction) => {
                ("Unknown Identifier", instruction.as_str().to_owned())
            }
            ReduceError::TypeError(t) => ("Type Error", t.as_str().to_owned()),
            ReduceError::UnexpectedArgument {
                instruction,
                arguments,
                expected: _,
                found: _,
            } => (
                "Unexpected Argument",
                format!(
                    "\"{}\" for \"{}\"",
                    arguments[0].as_str().to_owned(),
                    instruction.as_str().to_owned(),
                ),
            ),
            ReduceError::ExpectedArgument {
                instruction,
                expected,
                found,
            } => (
                "Expected Argument",
                format!(
                    "({} missing) for \"{}\"",
                    expected - found,
                    instruction.as_str().to_owned()
                ),
            ),
            ReduceError::ExpectedType {
                argument: _,
                expected,
                found: _,
            } => {
                let expected_types = expected.join(" ");
                ("Expected Type(s)", expected_types.as_str().to_owned())
            }
            ReduceError::LabelRedeclaration { label } => ( /* todo: tell redeclaration line */
                "Label Redeclaration. Already declared",
                label.as_str().to_owned(),
            ),
            ReduceError::InvalidLabel { label } => {
                ("Invalid Label", label.as_str().to_owned())
            }
        };

        f.write_fmt(format_args!("{} {} at line {}", msg, what, self.line))
    }
}

impl<'a> AsmError<'a> {
    pub fn from_reduce_error(err: ReduceError<'a>, input: &str) -> Self {
        let (span, expected, found) = match err {
            ReduceError::UnknownInstruction(ref pair) => {
                (pair.as_span(), None, None)
            }
            ReduceError::UnknownIdentifier(ref pair) => {
                (pair.as_span(), None, None)
            }
            ReduceError::TypeError(ref pair) => (pair.as_span(), None, None),
            ReduceError::UnexpectedArgument {
                ref instruction,
                arguments: _,
                expected,
                found,
            } => (instruction.as_span(), Some(expected), Some(found)),
            ReduceError::ExpectedArgument {
                ref instruction,
                expected,
                found,
            } => (instruction.as_span(), Some(expected), Some(found)),
            ReduceError::ExpectedType {
                ref argument,
                expected: _,
                found: _,
            } => (argument.as_span(), None, None),
            ReduceError::LabelRedeclaration { label: ref pair } => {
                (pair.as_span(), None, None)
            }
            ReduceError::InvalidLabel { label: ref pair } => {
                (pair.as_span(), None, None)
            }
        };

        /* find where the error happened (line number) */
        let line_text = span.get_input();
        let offset = line_text.as_ptr() as usize - input.as_ptr() as usize;
        let line =
            (input[..offset].bytes().filter(|&b| b == b'\n').count() + 1)
                as u32;

        AsmError {
            err,
            line,
            expected,
            found,
        }
    }
}

pub fn parse<'i>(
    ctx: &mut Context,
    input: &'i str,
) -> Result<Box<[u16]>, AsmError<'i>> {
    let mut result: Vec<_> = input
        .lines()
        .filter_map(|line| parse_line(line))
        .flatten()
        .collect();

    ctx.address = 0;

    result = result
        .into_iter()
        .filter_map(|statement| statement.reduce(ctx).transpose())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| AsmError::from_reduce_error(err.clone(), input))?;

    let alloc_offset = ctx.address;

    ctx.set_allocation_offset(alloc_offset);

    loop {
        ctx.address = 0;

        result = result
            .into_iter()
            .filter_map(|statement| statement.reduce(ctx).transpose())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| AsmError::from_reduce_error(err.clone(), input))?;

        if ctx.counter == 0 {
            break;
        }

        ctx.counter -= 1;
    }

    let mut data = Box::new([0u16; 0x10000]);

    result
        .iter()
        .fold(0, |acc, statement| statement.copy(data.as_mut_slice(), acc));

    Ok(data)
}

pub struct Assembly {
    data: Box<[u16]>,
    symbols: HashMap<String, Option<usize>>,
}

impl Assembly {
    pub fn symbols(&self) -> String {
        let mut buffer = String::new();

        let _ = self
            .symbols
            .keys()
            .zip(self.symbols.values())
            .filter_map(|(key, value)| value.map(|value| (key, value)))
            .try_for_each(|(key, value)| {
                buffer.write_fmt(format_args!("{key} = {value:#x}\n"))
            });

        buffer
    }

    pub fn binary(&self) -> Vec<u16> {
        self.data.to_vec()
    }

    pub fn mif(&self) -> String {
        mif::Mif::new(&self.data, mif::Radix::Hex, mif::Radix::Bin).to_string()
    }
}

pub fn assemble(
    entry: impl AsRef<Path>,
    syntax: impl AsRef<Path>,
) -> Result<Assembly, String> {
    let entry = std::fs::read_to_string(entry).unwrap();
    let syntax = std::fs::read_to_string(syntax).unwrap();

    assemble_from_buf(entry, syntax)
}

pub fn assemble_from_buf(
    input: impl AsRef<str>,
    syntax: impl AsRef<str>,
) -> Result<Assembly, String> {
    let is = cis::InstructionSet::from_str(syntax.as_ref())
        .map_err(|err| err.to_string())?;

    let (result, symbols) = {
        let mut ctx = Context::new(&is, 100);

        (parse(&mut ctx, input.as_ref()), ctx.labels)
    };

    let data = result.map_err(|err| err.to_string())?;

    Ok(Assembly { data, symbols })
}
