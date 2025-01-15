use assembler::assemble;
use clap::{value_parser, Parser};
use clio::{Input, Output};
use fs::Fs;

#[derive(Parser)]
#[command(name = "Assembler", about = "Assembler")]
struct Cli {
    #[arg(
        short,
        long,
        value_name = "input file path",
        help = "Input file",
        required = true,
        value_parser = value_parser!(Input).exists().is_file()
    )]
    input: Input,
    #[arg(
        short,
        long,
        value_name = "output file path",
        help = "Output file, if not provided, stdout will be used",
        default_value = "-"
    )]
    output: Output,
    #[arg(
        short,
        long,
        value_name = "synthax file path",
        help = "Synthax file",
        required = true,
        value_parser = value_parser!(Input).exists().is_file()
    )]
    synthax: Input,
}

fn main() {
    let mut cli = Cli::parse();

    let mut fs = Fs::new();

    // TODO: Implement reading from stdin
    // let mut buffer = Vec::new();

    // let mut input_reader = cli.input.lock();
    // input_reader.read_to_end(&mut buffer).unwrap();
    // fs.write("entry.asm", &buffer).unwrap();
    // buffer.clear();

    // let mut synthax_reader = cli.synthax.lock();
    // synthax_reader.read_to_end(&mut buffer).unwrap();
    // fs.write("synthax.toml", &buffer).unwrap();
    // buffer.clear();

    assemble(
        &mut fs,
        cli.input.path().to_str().unwrap(),
        cli.synthax.path().to_str().unwrap(),
    )
    .map(|assembly| {
        let mut output_writer = cli.output.lock();
        output_writer.write(assembly.mif().as_bytes()).unwrap();
    })
    .unwrap();
}
