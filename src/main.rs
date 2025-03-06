use std::{ffi::OsString, fs::File};

fn main() -> std::io::Result<()> {
    for fp in std::env::args_os() {
        match std::fs::exists(fp.clone()) {
            Ok(true) => continue,
            _ => create_file(fp)?,
        }
    }

    Ok(())
}

fn create_file(filepath: OsString) -> std::io::Result<()> {
    match File::create(filepath) {
        Ok(_) => {}
        Err(err) => println!("got error: {}", err),
    };

    Ok(())
}
