use rivik_assets::{formats::misc::Txt, load, lump::Lump, AssetLoadError};
use snafu::Report;

fn main() {
    println!(
        "{}",
        Report::capture(|| -> Result<(), AssetLoadError> {
            let cache = Lump::precache("test.lump");
            load("lump:test.lump#assets/cube.obj", Txt)?;
            load("lump:test.lump?#assets/cube.obj", Txt)?;
            Ok(())
        })
    );
}
