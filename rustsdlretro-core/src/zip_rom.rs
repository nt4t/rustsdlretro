use std::io::{self, Cursor};
use std::path::{Path, PathBuf};

/// Common ROM file extensions that may be found in a ZIP archive.
const ROM_EXTENSIONS: &[&str] = &[
    // NES
    ".nes",
    // SNES
    ".sfc", ".smc",
    // Genesis/Mega Drive
    ".md", ".smd", ".gen",
    // Game Boy / Color
    ".gb", ".gbc",
    // GBA
    ".gba",
    // N64
    ".n64", ".z64", ".v64",
    // Sega CD
    ".cue", ".iso", ".chd",
    // TurboGrafx-16/PC Engine
    ".pce",
];

/// RAII guard that removes a file on drop.
pub struct TempFileGuard {
    path: PathBuf,
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Result of ZIP ROM extraction.
pub enum ZipRomResult {
    /// In-memory data (for cores that don't need fullpath).
    Data {
        rom_data: Vec<u8>,
        filename: String,
    },
    /// Extracted to temp file (for cores that need fullpath).
    TempFile {
        path: PathBuf,
        _guard: Box<TempFileGuard>,
    },
}

/// Check if the given path is a ZIP file (by extension or magic bytes).
pub fn is_zip(path: &Path) -> bool {
    // Quick check by extension
    if let Some(ext) = path.extension() {
        if ext.to_string_lossy().to_lowercase() == "zip" {
            return true;
        }
    }

    // Check magic bytes as fallback
    if let Ok(file_data) = std::fs::read(path) {
        if file_data.len() >= 4 && &file_data[..2] == b"PK" {
            return true;
        }
    }

    false
}

/// Find the best ROM entry in a ZIP archive.
/// Returns (filename, index_in_archive).
fn find_rom_entry(data: &[u8]) -> io::Result<Option<(String, usize)>> {
    let cursor = Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;

    // First pass: find ROM files by extension
    let mut rom_indices: Vec<(String, usize)> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name().to_lowercase();

            // Skip directories and non-ROM files
            if name.ends_with('/') {
                continue;
            }

            for ext in ROM_EXTENSIONS {
                if name.ends_with(*ext) {
                    rom_indices.push((entry.name().to_string(), i));
                    break;
                }
            }
        }
    }

    match rom_indices.len() {
        0 => Ok(None),
        1 => Ok(Some(rom_indices.into_iter().next().unwrap())),
        // Multiple ROMs: pick the largest one
        _ => {
            let mut best_idx = rom_indices[0].1;
            let mut best_size = 0u64;

            for (name, idx) in &rom_indices {
                if let Ok(entry) = archive.by_index(*idx) {
                    if entry.size() > best_size {
                        best_size = entry.size();
                        best_idx = *idx;
                    }
                }
            }

            let filename = rom_indices.iter()
                .find(|(_, i)| *i == best_idx)
                .map(|(n, _)| n.clone())
                .unwrap_or_default();

            Ok(Some((filename, best_idx)))
        }
    }
}

/// Extract ROM data from a ZIP file into memory.
pub fn extract_zip_to_memory(path: &Path) -> Result<(Vec<u8>, String), String> {
    let file_data = std::fs::read(path).map_err(|e| format!("Failed to open ZIP: {}", e))?;

    // Check ZIP magic bytes
    if file_data.len() < 4 || &file_data[..2] != b"PK" {
        return Err("Not a valid ZIP file".to_string());
    }

    let cursor = Cursor::new(&file_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Invalid ZIP archive: {}", e))?;

    // Find the ROM entry
    let result = find_rom_entry(&file_data)
        .map_err(|e| format!("Failed to scan ZIP: {}", e))?;

    let (filename, idx) = result.ok_or("No ROM file found in ZIP archive")?;

    // Extract the selected entry
    let mut rom_file = archive.by_index(idx)
        .map_err(|e| format!("Failed to read ROM entry: {}", e))?;

    let mut rom_data = Vec::with_capacity(rom_file.size() as usize);
    use std::io::Read;
    rom_file.read_to_end(&mut rom_data)
        .map_err(|e| format!("Failed to extract ROM data: {}", e))?;

    if rom_data.is_empty() {
        return Err("Extracted ROM file is empty".to_string());
    }

    Ok((rom_data, filename))
}

/// Extract ROM from ZIP to a temp file on disk.
pub fn extract_zip_to_temp(path: &Path) -> Result<(PathBuf, Box<TempFileGuard>), String> {
    let file_data = std::fs::read(path).map_err(|e| format!("Failed to open ZIP: {}", e))?;

    // Check ZIP magic bytes
    if file_data.len() < 4 || &file_data[..2] != b"PK" {
        return Err("Not a valid ZIP file".to_string());
    }

    let cursor = Cursor::new(&file_data);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| format!("Invalid ZIP archive: {}", e))?;

    // Find the ROM entry
    let result = find_rom_entry(&file_data)
        .map_err(|e| format!("Failed to scan ZIP: {}", e))?;

    let (filename, idx) = result.ok_or("No ROM file found in ZIP archive")?;

    // Determine output extension from filename
    let ext = Path::new(&filename).extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("rom");

    // Create temp file with appropriate name
    let rom_stem = Path::new(&filename)
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("extracted_rom");

    let temp_path = std::env::temp_dir().join(format!("rustsdlretro_{}.{}", rom_stem, generate_id()));
    let full_temp_name = format!("{}.{}", temp_path.display(), ext);
    let full_temp_path = PathBuf::from(&full_temp_name);

    // Extract to file
    use std::io::{Read, Write};
    let mut out_file = std::fs::File::create(&full_temp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut rom_file = archive.by_index(idx)
        .map_err(|e| format!("Failed to read ROM entry: {}", e))?;

    io::copy(&mut rom_file, &mut out_file)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    let guard = Box::new(TempFileGuard {
        path: full_temp_path.clone(),
    });

    Ok((full_temp_path, guard))
}

/// Get a human-readable ROM name for display (from ZIP filename).
pub fn get_zip_rom_name(rom_path: &Path) -> String {
    rom_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Game")
        .to_string()
}

/// Generate a simple unique identifier (8 hex chars).
fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{:016x}", duration.as_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_zip_by_extension() {
        assert!(is_zip(Path::new("game.zip")));
        assert!(!is_zip(Path::new("game.sfc")));
        assert!(!is_zip(Path::new("/path/to/rom.nes")));
    }
}
