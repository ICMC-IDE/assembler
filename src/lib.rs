use std::{collections::HashMap, fmt::Write, path::Path, str::FromStr};

use asm::parse_line;
use ast::{Reduce, ReduceError};
use context::Context;

pub mod asm;
pub mod ast;
pub mod cis;
pub mod context;

pub fn parse<'i>(ctx: &mut Context, input: &'i str) -> Result<Box<[u16]>, ReduceError<'i>> {
    let mut result: Vec<_> = input
        .lines()
        .filter_map(|line| parse_line(line))
        .flatten()
        .collect();

    ctx.address = 0;

    result = result
        .into_iter()
        .filter_map(|statement| statement.reduce(ctx).transpose())
        .collect::<Result<Vec<_>, _>>()?;

    let alloc_offset = ctx.address;

    ctx.set_allocation_offset(alloc_offset);

    loop {
        ctx.address = 0;

        result = result
            .into_iter()
            .filter_map(|statement| statement.reduce(ctx).transpose())
            .collect::<Result<Vec<_>, _>>()?;

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
            .try_for_each(|(key, value)| buffer.write_fmt(format_args!("{key} = {value:#x}\n")));

        buffer
    }

    pub fn binary(&self) -> Vec<u16> {
        self.data.to_vec()
    }

    pub fn mif(&self) -> String {
        mif::Mif::new(&self.data, mif::Radix::Hex, mif::Radix::Bin).to_string()
    }
}

pub fn assemble(entry: impl AsRef<Path>, syntax: impl AsRef<Path>) -> Result<Assembly, String> {
    let entry = std::fs::read_to_string(entry).unwrap();
    let syntax = std::fs::read_to_string(syntax).unwrap();

    assemble_from_buf(entry, syntax)
}

pub fn assemble_from_buf(
    input: impl AsRef<str>,
    syntax: impl AsRef<str>,
) -> Result<Assembly, String> {
    let is = cis::InstructionSet::from_str(syntax.as_ref()).map_err(|err| err.to_string())?;

    let (result, symbols) = {
        let mut ctx = Context::new(&is, 100);

        (parse(&mut ctx, input.as_ref()), ctx.labels)
    };

    let data = result.map_err(|err| err.to_string())?;

    Ok(Assembly { data, symbols })
}
