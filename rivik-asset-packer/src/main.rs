//! Tool for generating packed lump files from a series of game assets.

use asset_packer::{pack, FastHash};
use clap::{arg, Parser};
use serde::{Deserialize, Serialize};
use snafu::ErrorCompat;

#[derive(Serialize, Deserialize)]
enum Entry {
    /// Data that is to be included verbatim in the lump file
    Buffer { path: String },
}

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    output: String,

    #[arg()]
    files: Vec<String>,
    // hash function selection
    #[arg(short = 'H', long, value_enum, default_value_t = FastHash::default())]
    hash: FastHash,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = pack(args.output, &args.files, &args.hash) {
        eprintln!("Error: {}", e);

        if e.iter_chain().skip(1).next().is_some() {
            eprintln!("\nCaused by:");
        }

        for (n, err) in e.iter_chain().skip(1).enumerate() {
            eprintln!("\t{n}: {err}");
        }

        if let Some(backtrace) = e.backtrace() {
            color_backtrace::BacktracePrinter::new()
                .print_trace(backtrace, &mut color_backtrace::default_output_stream())
                .unwrap();
        }
    }
}
