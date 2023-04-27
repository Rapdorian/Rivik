/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

use log::{error, info};
use rivik_assets::{formats::misc, load, Result};

fn run() -> Result<()> {
    env_logger::init();
    println!(
        "{:?}",
        load("bin:FOO.bin#C9C75B3BD976915C186D215629CDE656", misc::Bin)?
    );

    println!(
        "{:?}",
        load("bin:FOO.bin#5F33CFBE62D309BE3BFE94D0A2B52F9D", misc::Bin)?
    );

    println!(
        "{:?}",
        load("bin:FOO.bin#C9C75B3BD976915C186D215629CDE656", misc::Bin)?
    );
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        info!("{:?}", e);
    }
}
