use assembler::assemble;
use clap::{Parser, value_parser};
use clio::{Input, Output};

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
        value_name = "syntax file path",
        help = "Syntax file",
        required = true,
        value_parser = value_parser!(Input).exists().is_file()
    )]
    syntax: Input,
}

fn main() {
    let mut cli = Cli::parse();

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

    assemble(cli.input.path().path(), cli.syntax.path().path())
        .map(|assembly| {
            let mut output_writer = cli.output.lock();
            output_writer.write(assembly.mif().as_bytes()).unwrap();
        })
        .unwrap();
}
