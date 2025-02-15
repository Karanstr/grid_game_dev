#[cfg(not(target_arch = "wasm32"))]
use std::fs;

#[cfg(target_arch = "wasm32")]
use macroquad::miniquad;

pub trait FileSystem {
    fn read_file(path: &str) -> Option<String>;
    fn write_file(path: &str, contents: &str) -> bool;
}

#[cfg(not(target_arch = "wasm32"))]
pub struct NativeFS;

#[cfg(not(target_arch = "wasm32"))]
impl FileSystem for NativeFS {
    fn read_file(path: &str) -> Option<String> {
        fs::read_to_string(path).ok()
    }

    fn write_file(path: &str, contents: &str) -> bool {
        fs::write(path, contents).is_ok()
    }
}

#[cfg(target_arch = "wasm32")]
pub struct WebFS;

#[cfg(target_arch = "wasm32")]
impl FileSystem for WebFS {
    fn read_file(path: &str) -> Option<String> {
        miniquad::fs::load_file(path).ok().map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    }

    fn write_file(path: &str, contents: &str) -> bool {
        // For web, we'll handle saves differently
        // For now, return true to not break the interface
        true
    }
}

#[cfg(target_arch = "wasm32")]
pub use WebFS as FS;

#[cfg(not(target_arch = "wasm32"))]
pub use NativeFS as FS;
