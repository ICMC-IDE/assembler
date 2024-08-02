use std::{collections::HashMap, str::FromStr};

#[derive(serde_derive::Deserialize, Debug)]
pub struct Instruction {
    pub value: u32,
    pub length: usize,
    pub arguments: Vec<Argument>,
}

impl Instruction {
    pub fn argc(&self) -> usize {
        self.arguments
            .iter()
            .map(|arg| arg.index + 1)
            .max()
            .unwrap_or(0)
    }
}

#[derive(serde_derive::Deserialize, Debug)]
pub struct Argument {
    pub r#type: String,
    pub index: usize,
    pub offset: usize,
    pub length: usize,
}

impl Argument {
    #[inline(always)]
    pub fn format(&self, value: u32) -> u32 {
        (value as u32 & (1u32 << self.length).wrapping_sub(1)) << self.offset
    }
}

#[derive(serde_derive::Deserialize, Debug)]
pub struct InstructionSet {
    pub symbols: HashMap<String, Symbol>,
    pub instructions: HashMap<String, Vec<Instruction>>,
}

#[derive(serde_derive::Deserialize, Debug)]
pub struct Symbol {
    pub value: usize,
    pub tags: Vec<String>,
}

impl FromStr for InstructionSet {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str::<InstructionSet>(s)
    }
}
