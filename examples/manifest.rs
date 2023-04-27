/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::{env, fs::read_to_string};

use asset_packer::{compress::CompressionAlg, Entry, ManifestBuilder};

fn main() {
    if env::args().len() > 1 {
        for arg in env::args().skip(1) {
            // read this arg as a manifest and debug print it
            let manifest: ManifestBuilder = toml::from_str(&read_to_string(arg).unwrap()).unwrap();
            println!("{manifest:#?}");
        }
    } else {
        let manifest = ManifestBuilder::new().push(Entry::new("foo")).push(
            Entry::new("Bar.png")
                .rename("NewName")
                .compress(CompressionAlg::Lzma),
        );

        println!("{}", toml::to_string_pretty(&manifest).unwrap());
    }
}
