/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Tool for generating packed lump files from a series of game assets.

use std::fs;

use asset_packer::{
    manifest::{Entry, ManifestBuilder},
    FastHash,
};
use clap::{arg, Parser};
use snafu::ErrorCompat;

#[derive(Parser, Debug)]
struct Args {
    #[arg()]
    output: String,

    #[arg()]
    files: Vec<String>,
    // hash function selection
    #[arg(short = 'H', long, value_enum, default_value_t = FastHash::default())]
    hash: FastHash,

    #[arg(short, long)]
    manifest: Option<String>,
}

fn main() {
    let args = Args::parse();

    let manifest = match args.manifest {
        None => args.files.into_iter().map(|file| Entry::new(file)).fold(
            ManifestBuilder::new().with_hash(args.hash),
            |manifest, file| manifest.push(file),
        ),
        Some(path) => toml::from_str(&fs::read_to_string(path).unwrap()).unwrap(),
    };

    if let Err(e) = manifest.build(args.output) {
        eprintln!("Error: {}", e);

        if e.iter_chain().skip(1).next().is_some() {
            eprintln!("\nCaused by:");
        }

        for (n, err) in e.iter_chain().skip(1).enumerate() {
            eprintln!("\t{n}: {err}");
        }
    }
}
