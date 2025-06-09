use clap::{ArgGroup, Parser};

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
    output: String,
}

const BYTES: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

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

    let indices = encode(bytes);
    let len = indices.len();
    for idx in indices {
        print!("{}", BYTES[idx as usize] as char);
    }
    if len % 4 != 0 {
        for _ in 0..(4 - (len % 4)) {
            print!("=");
        }
    }
    println!();
}

fn encode(bytes: Vec<u8>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    for chunk in bytes.chunks(3) {
        let chunk_size = chunk.len();
        if chunk_size == 3 {
            result.push(chunk[0] >> 2);
            result.push((chunk[0] & 0x3) << 4 | ((chunk[1] & 0xF0) >> 4));
            result.push((chunk[1] & 0x0F) << 2 | ((chunk[2] & 0xC0) >> 6));
            result.push((chunk[2] << 2) >> 2);
        } else if chunk_size == 2 {
            result.push(chunk[0] >> 2);
            result.push((chunk[0] & 0x3) << 4 | ((chunk[1] & 0xF0) >> 4));
            result.push((chunk[1] & 0x0F) << 2);
        } else {
            result.push(chunk[0] >> 2);
            result.push((chunk[0] & 0x3) << 4);
        }
    }
    result
}
