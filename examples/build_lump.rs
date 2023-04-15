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
