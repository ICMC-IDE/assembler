use std::{collections::HashMap, error::Error, fmt::Display};

pub use crate::cis::*;

#[derive(Debug)]
pub struct Context<'is, 'a> {
    pub is: &'is InstructionSet,
    pub labels: HashMap<String, Option<usize>>,
    pub allocations: HashMap<String, usize>,
    pub address: usize,
    pub path: Vec<&'a str>,
    pub allocation_offset: Option<usize>,
}

#[derive(Debug)]
pub enum LabelError {
    InvalidLabel,
    Unavailable,
}

impl<'is, 'a> Context<'is, 'a> {
    pub fn new(is: &'is InstructionSet) -> Self {
        Self {
            is,
            labels: HashMap::new(),
            allocations: HashMap::new(),
            address: 0,
            path: Vec::new(),
            allocation_offset: None,
        }
    }

    pub fn set_allocation_offset(&mut self, mut offset: usize) {
        for (name, size) in self.allocations.drain() {
            self.labels.insert(name, Some(offset));
            offset += size;
        }

        self.allocation_offset = Some(offset);
    }

    pub fn advance(&mut self, len: usize) {
        self.address += len;
    }

    pub fn get_path(&self, label: &'a str) -> Result<String, LabelError> {
        if label.starts_with('.') {
            let levels = label.chars().take_while(|chr| chr == &'.').count();
            if levels == self.path.len() {
                let mut path = self.path.join(".");

                path.push('.');
                path.push_str(&label[levels..]);

                Ok(path)
            } else {
                Err(LabelError::InvalidLabel)
            }
        } else {
            Ok(label.to_owned())
        }
    }

    pub fn register_label(
        &mut self,
        label: &str,
        preregistered: bool,
    ) -> Result<usize, LabelError> {
        let path = self.get_path(label)?;
        let is_new = preregistered || !self.labels.contains_key(&path);

        if is_new {
            self.labels.insert(path, Some(self.address));
            Ok(self.address)
        } else {
            Err(LabelError::Unavailable)
        }
    }

    pub fn allocate(
        &mut self,
        label: &str,
        size: Option<usize>,
        preregistered: bool,
    ) -> Result<(), LabelError> {
        let path = self.get_path(label)?;

        let address = self.allocation_offset;

        if preregistered {
            if let Some(size) = size {
                if let Some(offset) = self.allocation_offset {
                    self.allocation_offset = Some(offset + size);
                } else {
                    self.allocations.insert(path, size);
                }
            }

            Ok(())
        } else if !self.labels.contains_key(&path) {
            self.labels.insert(path.clone(), address);

            if let Some(size) = size {
                if let Some(offset) = self.allocation_offset {
                    self.allocation_offset = Some(offset + size);
                } else {
                    self.allocations.insert(path, size);
                }
            }

            Ok(())
        } else {
            Err(LabelError::Unavailable)
        }
    }
}

impl Display for LabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:#?}"))
    }
}

impl Error for LabelError {}
