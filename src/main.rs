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

const BYTES: &[u8; 65] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
const PADDING_IDX: u32 = 64;
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
    let chars: String = indices
        .into_iter()
        .flat_map(|n| n.to_be_bytes())
        .map(|b| BYTES[b as usize] as char)
        .collect();

    println!("{}", chars);
}

fn encode(bytes: Vec<u8>) -> Vec<u32> {
    let mut result: Vec<u32> = Vec::new();
    for chunk in bytes.chunks(3) {
        let chunk_size = chunk.len();
        if chunk_size == 3 {
            let val = ((chunk[0] >> 2) as u32) << 24
                | (((chunk[0] & 0x3) << 4 | ((chunk[1] & 0xF0) >> 4)) as u32) << 16
                | (((chunk[1] & 0x0F) << 2 | ((chunk[2] & 0xC0) >> 6)) as u32) << 8
                | ((chunk[2] << 2) >> 2) as u32;
            result.push(val);
        } else if chunk_size == 2 {
            let val = ((chunk[0] >> 2) as u32) << 24
                | (((chunk[0] & 0x3) << 4 | ((chunk[1] & 0xF0) >> 4)) as u32) << 16
                | (((chunk[1] & 0x0F) << 2) as u32) << 8
                | PADDING_IDX;
            result.push(val);
        } else {
            let val = ((chunk[0] >> 2) as u32) << 24
                | (((chunk[0] & 0x3) << 4) as u32) << 16
                | PADDING_IDX << 8
                | PADDING_IDX;
            result.push(val);
        }
    }
    result
}
