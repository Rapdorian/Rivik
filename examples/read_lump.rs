/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use rivik_assets::{
    formats::misc::{Bin, Txt},
    load,
    lump::Lump,
    AssetLoadError,
};
use snafu::Report;

fn main() {
    println!(
        "{}",
        Report::capture(|| -> Result<(), AssetLoadError> {
            let cache = Lump::precache("track.bin");
            load("lump:track.bin#test", Bin)?;
            println!("{}", load("lump:track.bin?brotli#cube", Txt)?);
            Ok(())
        })
    );
}
