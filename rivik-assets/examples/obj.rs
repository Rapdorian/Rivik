/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::env;

use rivik_assets::{formats::mesh::ObjScene, load, AssetLoadError};
use snafu::ErrorCompat;

fn run() -> Result<(), AssetLoadError> {
    let mut args = env::args().skip(1);
    // grab asset uri
    if let Some(uri) = args.next() {
        let obj = load(&uri, ObjScene)?;
        // print contents of obj file
        for mesh in &obj.nodes {
            println!("{}", mesh.1);
            for v in &mesh.0.verts {
                println!("\t{},{},{}", v.x, v.y, v.z);
            }
        }
    } else {
        eprintln!("Missing argument: Filename URI");
    }

    Ok(())
}

fn main() {
    env_logger::init();

    if env::var("RUST_BACKTRACE").is_err() {
        env::set_var("RUST_BACKTRACE", "1");
    }
    //color_eyre::install().expect("eyre failed to init");
    color_backtrace::install();

    if let Err(e) = run() {
        eprintln!("Error: {}", e);

        if e.iter_chain().skip(1).next().is_some() {
            eprintln!("\nCaused by:");
        }

        for (n, err) in e.iter_chain().skip(1).enumerate() {
            eprintln!("\t{n}: {err}");
        }

        if let Some(backtrace) = e.backtrace() {
            color_backtrace::BacktracePrinter::new()
                .add_frame_filter(Box::new(color_backtrace::default_frame_filter))
                .print_trace(backtrace, &mut color_backtrace::default_output_stream())
                .unwrap();
        }
    }
}
