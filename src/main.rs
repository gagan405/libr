use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file: String,
    #[arg(short, long)]
    output: String,
}

fn main() {
    let args = Args::parse();
    println!("Hello {} -> {}!", args.file, args.output);
}
