extern crate regex;
use regex::Regex;
use std::{
    env,
    fs::{self, read_dir},
    io::{Read, Write},
    path::{Path, PathBuf},
};

/// [`recurse_files`] will traverse a directory and find all
/// file paths within that directory, then return the resulting
/// file paths as a `std::io:Result<Vec<String>>`, after which
/// [`core::option::Option::unwrap_or_else`] can be used to
/// retrieve the `Vec<String>`.
///
/// # example
/// [`recurse_files`] can be used to gather a list of all
/// file paths in a directory, including subdirectories, and
/// return that list of files:
/// ```rust
/// fn get_file_paths(path_to_recurse: impl AsRef<Path>) -> std::io::Result<Vec<String>> {
///     recurse_files(path_to_recurse).unwrap_or_else(|_| {
///         panic!("could not recurse through the {directory_to_recurse} directory")
///     });
/// }
/// ```
fn recurse_files(user_path: impl AsRef<Path>) -> std::io::Result<Vec<String>> {
    let mut buf = vec![];
    println!("{:#?}", user_path.as_ref());
    let absolute_path = PathBuf::from(user_path.as_ref());
    println!("{:#?}", absolute_path);
    let entries = read_dir(absolute_path)?;
    for entry in entries {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_dir() {
            let mut subdir = recurse_files(entry.path())?;
            buf.append(&mut subdir);
        }
        if meta.is_file() {
            buf.push(entry.path().to_str().unwrap().to_string());
        }
    }
    Ok(buf)
}

/// [`minify`] will apply regex rules to files to reduce
/// file size and enable multi-file development. it
/// will normalize spaces, remove comments, remove
/// unnecessary semi-colons, and trim spaces where
/// appropriate.
///
/// # notes
/// in the future, the `extension` parameter that is
/// passed in [`minify_files`], which calls this, will
/// be used to determine which rules to fill
/// `patterns_and_replacement` with.
///
/// # example
/// [`minify`] can be used to combine and minify the content
/// from a `Vec<String>` `file_paths_to_minify`, then return
///  that content as a [`String`]:
/// ```rust
/// fn minify_file_paths(file_paths_to_minify: Vec<String>) -> String {
///     file_paths_to_minify
///         .iter()
///         .map(|file_path| {
///             fs::OpenOptions::new()
///                 .read(true)
///                 .open(file_path)
///                 .map(|mut file| minify(&mut file))
///                 .unwrap()
///         })
///         .collect::<String>()
/// }
/// ```
fn minify(file: &mut std::fs::File) -> String {
    let mut string_buffer = String::new();
    // note: these currently work best with CSS
    let patterns_and_replacement = [
        (Regex::new(r"\s+").unwrap(), " "),
        (Regex::new(r"; }").unwrap(), "}"),
        (Regex::new(r"([,:;\{\}>])\s").unwrap(), "${1}"),
        (Regex::new(r"\s([,:;\{\}>])").unwrap(), "${1}"),
        (Regex::new(r"0 0 0 0").unwrap(), "0"),
        (Regex::new(r"/\*.*?\*/").unwrap(), ""),
    ];

    let _ = file.read_to_string(&mut string_buffer);
    for pattern in patterns_and_replacement {
        string_buffer = pattern.0.replace_all(&string_buffer, pattern.1).to_string()
    }
    string_buffer
}

/// [`minify_files`] combines all files of type `extension`
/// within the `./assets/` directory in the `destination_file`
/// **in-place**, and doesn't return anything.
///
/// # notes
/// the file referenced by `destination_file_path` must exist
/// on-disk before the build process is started, otherwise this
/// function won't be able to open the file in truncated/write
/// mode.
///
/// # example
/// [`minify_files`] can be used to combine and minify
/// the content from all css files, and put their contents
/// into `./assets/css/style.css`:
/// ```rust
/// fn minify_function() {
///     minify_files("css", "./assets/css", "style.css");
/// }
/// ```
fn minify_files(extension: &str, destination_folder_path: &str, destination_file_name: &str) {
    let destination_file_path = &format!("{destination_folder_path}/{destination_file_name}");
    println!("{}", destination_file_path);
    let files_to_minify = recurse_files(destination_folder_path).unwrap_or_else(|_| {
        panic!(
            "could not open {} directory to minify {} files",
            destination_folder_path, extension
        )
    });
    let mut destination_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(destination_file_path)
        .unwrap_or_else(|_| {
            panic!(
                "could not open destination file ({}).",
                destination_file_path
            )
        });
    let files_without_destination_file = files_to_minify
        .iter()
        .filter(|file| file.ends_with(extension) && !file.contains(destination_file_path))
        .collect::<Vec<_>>();
    let minified_file_content: String = files_without_destination_file
        .iter()
        .map(|file_path| {
            fs::OpenOptions::new()
                .read(true)
                .open(file_path)
                .map(|mut file| minify(&mut file))
                .unwrap()
        })
        .collect::<String>();
    let _ = destination_file.write_all(minified_file_content.as_bytes());
}

/// [`main`] is the entry point for the rcss minification program.
///
/// # examples
/// `cargo run -- c:\some-dir\css`: will take all css files in the `c:\some-dir\css` path, and
/// combine them into a new `c:\some-dir\css\style.css` file.
///
/// `cargo run -- c:\some-dir\css new-style.css`: will take all css files in the  `c:\some-dir\css`
/// path, and combine them into a new `c:\some-dir\css\new-style.css` file.
fn main() {
    let args: Vec<String> = env::args().collect();
    let css_folder = &args[1];
    let destination_file = if args.len() > 2 { &args[2] } else { "" };
    let default_destination_file = "style.css";

    // throw if the directory argument is empty
    assert!(!css_folder.is_empty());

    // use our `default_destination_file` if no `destination_file` was provided
    if destination_file.is_empty() {
        minify_files("css", css_folder, default_destination_file);
        return;
    }

    minify_files("css", css_folder, destination_file);
}
