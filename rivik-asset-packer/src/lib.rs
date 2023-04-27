/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

//! Tool for generating packed lump files from a series of game assets.

pub mod compress;
pub mod manifest;
pub(crate) mod pack;

pub use manifest::Entry;
pub use manifest::ManifestBuilder;
pub use pack::FastHash;
