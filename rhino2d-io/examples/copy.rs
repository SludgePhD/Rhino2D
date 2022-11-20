//! Deserializes a model, and serializes it to a different file.
//!
//! The resulting copy should be semantically indistinguishable from the original: it should render
//! the same, behave the same, have the same metadata, and the same textures. However, due to
//! differences in JSON encoding, it is unlikely to be byte-for-byte identical with the original.

use std::io;

use rhino2d_io::InochiPuppet;

fn main() -> io::Result<()> {
    env_logger::Builder::new()
        .filter_module(env!("CARGO_CRATE_NAME"), log::LevelFilter::Debug)
        .init();

    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    match &*args {
        [in_path, out_path] => {
            let puppet = InochiPuppet::from_path(in_path)?;
            puppet.save(out_path)?;
            if let Err(e) = InochiPuppet::from_path(out_path) {
                eprintln!("failed to read written file: {}", e);
            }
            Ok(())
        }
        _ => {
            eprintln!("usage: copy <input-path> <output-path>");
            std::process::exit(1);
        }
    }
}
