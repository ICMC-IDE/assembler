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
        (value & (1u32 << self.length).wrapping_sub(1)) << self.offset
    }
}

#[derive(serde_derive::Deserialize, Debug)]
pub struct InstructionSet {
    pub symbols: HashMap<String, Symbol>,
    pub instructions: HashMap<String, Vec<Instruction>>,
}

impl InstructionSet {
    pub fn get_symbol(&self, name: &str) -> Option<&Symbol> {
        // FIXME: this allocated everytime
        let name = name.to_ascii_lowercase();
        self.symbols.get(&name)
    }

    pub fn get_instruction(&self, name: &str) -> Option<&[Instruction]> {
        // FIXME: this allocated everytime
        let name = name.to_ascii_lowercase();
        self.instructions.get(&name).map(std::ops::Deref::deref)
    }
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
