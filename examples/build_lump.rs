/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use asset_packer::{pack, FastHash, PackError};
use snafu::Report;

fn main() {
    if let Err(e) = Report::capture_into_result(|| {
        // build a test lump file
        pack(
            "test.lump",
            &["assets/cube.obj", "assets/fighter.albedo.png"],
            &FastHash::Sea,
        )
    }) {
        eprintln!("{e}");
    }
}
