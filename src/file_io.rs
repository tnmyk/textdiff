#[cfg(not(target_family = "wasm"))]
use std::path::Path;

/// Files larger than this are rejected before reading their contents.
pub const MAX_FILE_BYTES: usize = 10 * 1024 * 1024;

/// A validated UTF-8 text file, ready to place in an editor pane.
#[derive(Clone, Debug)]
pub struct LoadedText {
    pub name: String,
    pub text: String,
    pub bytes: usize,
}

impl LoadedText {
    /// Validate an in-memory byte string, rejecting binaries, invalid UTF-8,
    /// and oversized input.
    pub fn from_bytes(name: &str, bytes: Vec<u8>) -> Result<Self, String> {
        Self::decode(display_name(name).to_string(), bytes)
    }

    /// Validate text that already arrived as a string, usually from the web
    /// file bridge.
    #[cfg_attr(not(target_family = "wasm"), allow(dead_code))]
    pub fn from_text(name: &str, text: String) -> Result<Self, String> {
        Self::from_bytes(name, text.into_bytes())
    }
}

/// Read and validate a text file from disk.
#[cfg(not(target_family = "wasm"))]
pub fn read_text_file(path: &Path) -> Result<LoadedText, String> {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string());

    let metadata =
        std::fs::metadata(path).map_err(|error| format!("Could not open {name}: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("{name} is not a file."));
    }

    let size = usize::try_from(metadata.len()).unwrap_or(usize::MAX);
    check_size(&name, size)?;

    let bytes = std::fs::read(path).map_err(|error| format!("Could not read {name}: {error}"))?;
    LoadedText::from_bytes(&name, bytes)
}

impl LoadedText {
    fn decode(name: String, bytes: Vec<u8>) -> Result<Self, String> {
        let name = display_name(&name).to_string();
        check_size(&name, bytes.len())?;

        if bytes.contains(&0) {
            return Err(format!(
                "{name} looks like a binary file. Choose a UTF-8 text file."
            ));
        }

        let bytes_len = bytes.len();
        let text =
            String::from_utf8(bytes).map_err(|_| format!("{name} is not valid UTF-8 text."))?;
        let text = match text.strip_prefix('\u{feff}') {
            Some(text) => text.to_string(),
            None => text,
        };

        Ok(Self {
            name,
            text,
            bytes: bytes_len,
        })
    }
}

fn display_name(name: &str) -> &str {
    if name.is_empty() {
        "The selected file"
    } else {
        name
    }
}

fn check_size(name: &str, size: usize) -> Result<(), String> {
    if size > MAX_FILE_BYTES {
        return Err(format!(
            "{name} is too large ({}). The limit is 10 MB.",
            format_size(size)
        ));
    }
    Ok(())
}

/// Human-readable byte count, e.g. `512 B`, `12.4 KB`, `3.2 MB`.
pub fn format_size(bytes: usize) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;

    let bytes = bytes as f64;
    if bytes < KB {
        format!("{bytes:.0} B")
    } else if bytes < MB {
        format!("{:.1} KB", bytes / KB)
    } else {
        format!("{:.1} MB", bytes / MB)
    }
}

#[cfg(target_family = "wasm")]
pub mod web {
    use wasm_bindgen::{JsCast as _, JsValue};

    /// Ask the page to open its hidden file input for the given pane.
    pub fn open_picker(pane: &str) -> Result<(), String> {
        let function = js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("diffOpenFile"))
            .map_err(|_| "The page file picker is unavailable.".to_string())?
            .dyn_into::<js_sys::Function>()
            .map_err(|_| "The page file picker is unavailable.".to_string())?;

        function
            .call1(&JsValue::NULL, &JsValue::from_str(pane))
            .map_err(|_| "The page file picker could not be opened.".to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_plain_text() {
        let loaded = LoadedText::from_text("note.txt", "hello\nworld\n".to_string()).unwrap();
        assert_eq!(loaded.name, "note.txt");
        assert_eq!(loaded.text, "hello\nworld\n");
        assert_eq!(loaded.bytes, 12);
    }

    #[test]
    fn strips_utf8_bom() {
        let bytes = b"\xEF\xBB\xBFhello".to_vec();
        let loaded = LoadedText::from_bytes("bom.txt", bytes).unwrap();
        assert_eq!(loaded.text, "hello");
        assert_eq!(loaded.bytes, 8);
    }

    #[test]
    fn rejects_nul_bytes_as_binary() {
        let error = LoadedText::from_bytes("image.png", b"text\0more".to_vec()).unwrap_err();
        assert!(error.contains("binary"), "unexpected error: {error}");
    }

    #[test]
    fn rejects_invalid_utf8() {
        let error = LoadedText::from_bytes("latin1.txt", vec![0xE9, 0x74, 0x65]).unwrap_err();
        assert!(error.contains("UTF-8"), "unexpected error: {error}");
    }

    #[test]
    fn rejects_oversized_input() {
        let error = check_size("huge.txt", MAX_FILE_BYTES + 1).unwrap_err();
        assert!(error.contains("too large"), "unexpected error: {error}");
        assert!(error.contains("10 MB"), "unexpected error: {error}");
    }

    #[test]
    fn formats_file_sizes() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(12 * 1024), "12.0 KB");
        assert_eq!(format_size(3 * 1024 * 1024), "3.0 MB");
    }

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn reads_a_file_from_disk() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "textdiff-file-io-{}-{unique}.txt",
            std::process::id()
        ));

        std::fs::write(&path, "first\nsecond\n").unwrap();
        let loaded = read_text_file(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded.text, "first\nsecond\n");
        assert_eq!(loaded.bytes, 13);
        assert!(loaded.name.starts_with("textdiff-file-io-"));
    }
}
