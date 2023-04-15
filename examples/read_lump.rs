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
