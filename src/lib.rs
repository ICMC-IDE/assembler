#![feature(iter_array_chunks)]

use fs::Fs;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Assembly {
    result: customasm::asm::AssemblyResult,
}

#[wasm_bindgen]
pub struct Assembler {}

struct FakeFs<'a> {
    fs: &'a Fs,
    filenames: Vec<String>,
}

impl<'a> From<&'a Fs> for FakeFs<'a> {
    fn from(value: &'a Fs) -> Self {
        Self {
            fs: value,
            filenames: value.files(),
        }
    }
}

impl<'a> customasm::util::FileServer for FakeFs<'a> {
    fn get_handle(
        &mut self,
        report: &mut customasm::diagn::Report,
        span: Option<customasm::diagn::Span>,
        filename: &str,
    ) -> Result<customasm::util::FileServerHandle, ()> {
        self.filenames
            .iter()
            .enumerate()
            .find(|(_index, name)| name.as_str() == filename)
            .map(|(index, _name)| index)
            .ok_or(())
    }

    fn get_filename(&self, file_handle: customasm::util::FileServerHandle) -> &str {
        &self.filenames[file_handle as usize]
    }

    fn get_bytes(
        &self,
        report: &mut customasm::diagn::Report,
        span: Option<customasm::diagn::Span>,
        file_handle: customasm::util::FileServerHandle,
    ) -> Result<Vec<u8>, ()> {
        let filename = self.get_filename(file_handle);

        self.fs.read(filename).map(Into::into).ok_or(())
    }

    fn write_bytes(
        &mut self,
        report: &mut customasm::diagn::Report,
        span: Option<customasm::diagn::Span>,
        filename: &str,
        data: &Vec<u8>,
    ) -> Result<(), ()> {
        self.fs.write(filename, data.as_ref());

        Ok(())
    }
}

#[wasm_bindgen]
impl Assembler {
    pub fn assemble(fs: &Fs, filenames: &str) -> Result<Assembly, String> {
        let opts = customasm::asm::AssemblyOptions::new();
        let filenames = filenames.split(':').collect::<Vec<_>>();
        let mut report = customasm::diagn::Report::new();

        let mut fileserver = FakeFs::from(fs);

        let result = customasm::asm::assemble(&mut report, &opts, &mut fileserver, &filenames);

        if result.error {
            let mut vec = Vec::new();
            report.print_all(&mut vec, &fileserver, true);
            return Err(String::from_utf8(vec).unwrap());
        }

        Ok(Assembly { result })
    }
}

#[wasm_bindgen]
impl Assembly {
    pub fn symbols(&self) -> String {
        let decls = self.result.decls.as_ref().unwrap();
        let defs = self.result.defs.as_ref().unwrap();

        decls
            .symbols
            .format(decls, defs, &mut |result, symbol_decl, name, bigint| {
                if let customasm::util::SymbolKind::Label = symbol_decl.kind {
                    result.push_str(name);
                    result.push_str(&format!(" = 0x{:x}\n", bigint));
                }
            })
    }

    pub fn binary(&self) -> Vec<u16> {
        self.result
            .output
            .as_ref()
            .unwrap()
            .format_binary()
            .into_iter()
            .array_chunks()
            .map(|bytes| u16::from_be_bytes(bytes))
            .collect()
    }

    pub fn mif(&self) -> String {
        self.result.output.as_ref().unwrap().format_mif()
    }
}
