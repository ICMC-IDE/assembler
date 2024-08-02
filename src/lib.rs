use std::{collections::HashMap, fmt::Write, str::FromStr};

use asm::parse_line;
use ast::{Reduce, ReduceError};
use context::Context;
use fs::Fs;
use wasm_bindgen::prelude::*;

pub mod asm;
pub mod ast;
pub mod cis;
pub mod context;

pub fn parse<'c, 'i>(ctx: &'c mut Context, input: &'i str) -> Result<Box<[u16]>, ReduceError<'i>> {
    // dbg!(&ctx);

    let mut result: Vec<_> = input
        .lines()
        .enumerate()
        .filter_map(|(_num, line)| parse_line(line))
        .flatten()
        .collect();

    let mut i = 0;

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

        if i > 100 {
            break;
        }

        i += 1;
    }

    let mut data = Box::new([0u16; 0x10000]);

    result
        .iter()
        .fold(0, |acc, statement| statement.copy(data.as_mut_slice(), acc));

    Ok(data)
}

#[wasm_bindgen]
pub struct Assembly {
    data: Box<[u16]>,
    symbols: HashMap<String, Option<usize>>,
}

#[wasm_bindgen]
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

#[wasm_bindgen]
pub fn assemble(fs: &Fs, entry: &str, syntax: &str) -> Result<Assembly, String> {
    let syntax = fs.read(&syntax).unwrap();
    let input = fs.read(entry).unwrap();
    let is = cis::InstructionSet::from_str(&syntax).map_err(|err| err.to_string())?;

    let (result, symbols) = {
        let mut ctx = Context::new(&is);

        (parse(&mut ctx, &input), ctx.labels)
    };

    let data = result.map_err(|err| err.to_string())?;

    Ok(Assembly { data, symbols })
}
