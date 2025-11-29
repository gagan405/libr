use std::fs::write;
use clap::{ArgGroup, Parser};
use libgb::encode_to_bytes;

#[derive(Parser, Debug)]
#[command(group(
    ArgGroup::new("input")
        .required(true)
        .args(&["file", "raw_string"]),
))]
struct Args {
    /// Input file
    #[arg(short, long)]
    file: Option<String>,

    /// Raw string input
    #[arg(short = 'r', long)]
    raw_string: Option<String>,

    /// Output file
    #[arg(short, long)]
    output: Option<String>,
}

const BYTES: &[u8; 65] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
fn main() {
    let args = Args::parse();

    let bytes = if let Some(file_path) = args.file {
        std::fs::read(&file_path)
            .unwrap_or_else(|e| panic!("Failed to read file {}: {}", file_path, e))
    } else if let Some(raw) = args.raw_string {
        raw.as_bytes().to_vec()
    } else {
        unreachable!("clap ensures either `file` or `raw_string` is present");
    };

    let indices = encode_to_bytes(bytes);
    let chars: String = indices
        .into_iter()
        .flat_map(|n| n.to_be_bytes())
        .map(|b| BYTES[b as usize] as char)
        .collect();

    if let Some(output) = args.output {
        write(output, chars.as_bytes()).expect("Failed to write output");
    } else {
        println!("{}", chars);
    }
}

