use std::path::PathBuf;

pub fn get_file_path<F: AsRef<str>>(f: F) -> PathBuf {
    let mut path: PathBuf = env!("CARGO_TARGET_TMPDIR").into();
    path.push(f.as_ref());
    path
}
