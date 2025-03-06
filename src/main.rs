use std::{
    ffi::OsString,
    fs::{self, File},
    io::{self, ErrorKind},
    path::Path,
};

fn main() -> io::Result<()> {
    for fp in std::env::args_os().skip(1) {
        if let Ok(true) = fs::exists(fp.clone()) {
            continue;
        }

        create_file(fp)?
    }

    Ok(())
}

fn create_file(filepath: OsString) -> io::Result<()> {
    // if the input `filepath` ends with `/` (eg. `foo/bar/baz/`)
    // then create directory instead of file
    if filepath.to_str().unwrap().ends_with("/") {
        return fs::create_dir_all(filepath);
    }

    match File::create(filepath.clone()) {
        Ok(_) => {}
        Err(err) => {
            // the parent directory doesn't exists
            if err.kind() == ErrorKind::NotFound {
                // get the parent
                let parent_dir = Path::new(&filepath).parent().unwrap();
                fs::create_dir_all(parent_dir)?;

                return create_file(filepath);
            }
        }
    };

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn create_file() {
        todo!();
    }
}
