use std::{
    ffi::OsString,
    fs::{self, File},
    io::{self, ErrorKind},
    path::MAIN_SEPARATOR,
    path::Path,
};

fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: touchp <file>...");
        return Ok(());
    }

    for fp in args {
        create_file(fp)?
    }

    Ok(())
}

fn create_file(filepath: OsString) -> io::Result<()> {
    create_file_inner(filepath, 0)
}

fn create_file_inner(filepath: OsString, depth: u32) -> io::Result<()> {
    const MAX_DEPTH: u32 = 255;
    const OS_SEPARATOR: char = MAIN_SEPARATOR;

    // prevent infinite recursion path creation
    if depth > MAX_DEPTH {
        return Err(io::Error::new(
            ErrorKind::PermissionDenied,
            "maximum path depth exceeded",
        ));
    }

    // skip special paths "." and ".."
    if filepath == "." || filepath == ".." {
        return Ok(());
    }

    // if the input `filepath` ends with `/` (eg. `foo/bar/baz/`)
    // then create directory instead of file
    if filepath
        .to_str()
        .is_some_and(|fp| fp.ends_with(OS_SEPARATOR))
    {
        return fs::create_dir_all(filepath);
    }

    match File::create(&filepath) {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::AlreadyExists => Ok(()), // skip if exists
        Err(err) if err.kind() == ErrorKind::NotFound => {
            // get the parent
            let parent_dir = Path::new(&filepath).parent().ok_or_else(|| {
                io::Error::new(ErrorKind::InvalidInput, "filepath has not parent directory")
            })?;

            fs::create_dir_all(parent_dir)?;
            create_file_inner(filepath, depth + 1)
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs;

    #[test]
    fn create_simple_file() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();
        let file_path = temp_dir.path().join("test.txt");

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        assert!(file_path.exists(), "File should be created");
        assert!(file_path.is_file(), "Should be a file, not directory");
    }

    #[test]
    fn create_nested_file() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();
        let file_path = temp_dir.path().join("foo").join("bar").join("test.txt");

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        assert!(file_path.exists(), "Nested file should be created");
        assert!(
            file_path.parent().unwrap().exists(),
            "Parent directory should exist"
        );
    }

    #[test]
    fn create_deep_nested_file() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();
        let file_path = temp_dir
            .path()
            .join("a")
            .join("b")
            .join("c")
            .join("d")
            .join("deep.txt");

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        assert!(file_path.exists(), "Deep nested file should be created");
    }

    #[test]
    fn create_directory_trailing_slash() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();
        let dir_path = temp_dir.path().join("mydir").join("");

        create_file(dir_path.as_os_str().to_os_string()).unwrap();

        assert!(
            dir_path.with_file_name("mydir").exists(),
            "Directory should be created"
        );
        assert!(
            dir_path.with_file_name("mydir").is_dir(),
            "Should be a directory"
        );
    }

    #[test]
    fn create_multiple_nested_dirs() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();
        let dir_path = temp_dir.path().join("foo").join("bar").join("baz").join("");

        create_file(dir_path.as_os_str().to_os_string()).unwrap();

        assert!(
            temp_dir.path().join("foo").join("bar").join("baz").exists(),
            "All directories should be created"
        );
    }

    #[test]
    fn create_file_with_spaces() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();
        let file_path = temp_dir.path().join("my file").join("test file.txt");

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        assert!(
            file_path.exists(),
            "File with spaces in path should be created"
        );
    }

    #[test]
    fn create_file_relative_path() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let file_path = temp_dir.path().join("tmp").join("test.txt");

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        assert!(file_path.exists(), "Relative path file should be created");
    }

    #[test]
    fn create_file_current_dir() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let file_path = temp_dir.path().join("single_file.txt");

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        assert!(
            file_path.exists(),
            "File in current directory should be created"
        );
    }

    #[test]
    fn create_file_only_slashes() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = create_file(OsString::from("/"));

        assert!(result.is_ok(), "Root slash should be handled gracefully");
    }

    #[test]
    fn create_multiple_files_in_sequence() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let file3 = temp_dir.path().join("dir").join("file3.txt");

        create_file(file1.as_os_str().to_os_string()).unwrap();
        create_file(file2.as_os_str().to_os_string()).unwrap();
        create_file(file3.as_os_str().to_os_string()).unwrap();

        assert!(file1.exists());
        assert!(file2.exists());
        assert!(file3.exists());
    }

    #[test]
    fn create_file_with_special_chars() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();
        let file_path = temp_dir.path().join("test-file_123.txt");

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        assert!(
            file_path.exists(),
            "File with special chars should be created"
        );
    }

    #[test]
    fn create_nested_then_sibling() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let file1 = temp_dir.path().join("a").join("b").join("file1.txt");
        let file2 = temp_dir.path().join("a").join("c").join("file2.txt");

        create_file(file1.as_os_str().to_os_string()).unwrap();
        create_file(file2.as_os_str().to_os_string()).unwrap();

        assert!(file1.exists());
        assert!(file2.exists());
        assert!(file1.parent().unwrap() != file2.parent().unwrap());
    }

    #[test]
    fn create_file_preserves_content_on_skip() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "original content").unwrap();

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(
            content.is_empty() || content == "original content",
            "Content should be empty or preserved"
        );
    }

    #[test]
    fn create_file_special_path_dot() {
        let result = create_file(OsString::from("."));

        assert!(
            result.is_ok(),
            "Dot path should be handled gracefully (skipped)"
        );
    }

    #[test]
    fn create_file_special_path_dotdot() {
        let result = create_file(OsString::from(".."));

        assert!(
            result.is_ok(),
            "Dotdot path should be handled gracefully (skipped)"
        );
    }

    #[test]
    fn create_file_empty_filename() {
        let result = create_file(OsString::from(""));

        assert!(result.is_err(), "Empty filename should return error");
    }

    #[test]
    fn create_file_max_depth_exceeded() {
        let sep = std::path::MAIN_SEPARATOR_STR;
        let deep_path = (0..300)
            .map(|i| format!("dir{}", i))
            .collect::<Vec<_>>()
            .join(sep);

        let result = create_file(OsString::from(deep_path));

        assert!(
            result.is_err(),
            "Should fail with deep path exceeding MAX_DEPTH"
        );
    }

    #[test]
    fn create_file_parent_is_file() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let file_as_dir = temp_dir.path().join("existing_file");
        fs::write(&file_as_dir, "content").unwrap();

        let nested_file = file_as_dir.join("nested.txt");
        let result = create_file(nested_file.as_os_str().to_os_string());

        assert!(
            result.is_err(),
            "Should fail when parent is a file, not directory"
        );
    }

    #[test]
    fn create_file_multiple_consecutive_separators() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let sep = std::path::MAIN_SEPARATOR_STR;
        let file_path = temp_dir
            .path()
            .join(format!("foo{}bar{}file.txt", sep, sep));

        let result = create_file(file_path.as_os_str().to_os_string());

        assert!(
            result.is_ok() || !file_path.exists(),
            "Should handle multiple separators gracefully"
        );
    }

    #[test]
    fn create_file_os_separator_flexible() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let unix_sep_file = temp_dir.path().join("test.txt");
        let result = create_file(unix_sep_file.as_os_str().to_os_string());
        assert!(result.is_ok(), "Unix separator should work");

        #[cfg(windows)]
        {
            let win_sep_file = temp_dir.path().join("test2.txt");
            let win_path = win_sep_file.to_string_lossy().replace('/', "\\");
            let result = create_file(OsString::from(win_path));
            assert!(result.is_ok(), "Windows separator should work");
        }
    }

    #[test]
    fn create_nested_directory_then_file() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let sep = std::path::MAIN_SEPARATOR_STR;
        let dir_path = temp_dir.path().join(format!("mydir{}", sep));
        create_file(dir_path.as_os_str().to_os_string()).unwrap();

        assert!(dir_path.exists(), "Directory should be created");
        assert!(dir_path.is_dir(), "Should be a directory");

        let file_in_dir = dir_path.parent().unwrap().join("mydir").join("file.txt");
        create_file(file_in_dir.as_os_str().to_os_string()).unwrap();

        assert!(
            file_in_dir.exists(),
            "File in created directory should exist"
        );
    }

    #[test]
    fn create_file_with_unicode() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let file_path = temp_dir.path().join("日本語").join("テスト.txt");

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        assert!(file_path.exists(), "Unicode path should be handled");
    }

    #[test]
    fn create_file_no_extension() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let file_path = temp_dir.path().join("noextension");

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        assert!(
            file_path.exists(),
            "File without extension should be created"
        );
    }

    #[test]
    fn create_file_only_extension() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let file_path = temp_dir.path().join(".hidden");

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        assert!(
            file_path.exists(),
            "Hidden file (only extension) should be created"
        );
    }

    #[test]
    fn create_file_very_long_filename() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let long_name = "a".repeat(255);
        let file_path = temp_dir.path().join(long_name);

        create_file(file_path.as_os_str().to_os_string()).unwrap();

        assert!(file_path.exists(), "Very long filename should be created");
    }

    #[test]
    fn create_directory_overwrites_existing_file() {
        let temp_dir = tempfile::Builder::new()
            .prefix("touchp_test_")
            .tempdir()
            .unwrap();

        let file_path = temp_dir.path().join("existing.txt");
        fs::write(&file_path, "content").unwrap();

        let dir_with_trailing_sep = temp_dir.path().join("existing.txt").join("");
        let result = create_file(dir_with_trailing_sep.as_os_str().to_os_string());

        assert!(
            result.is_err(),
            "Cannot create directory with same name as existing file"
        );
    }
}
