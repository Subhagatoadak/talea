use std::{fs, path::Path};

fn main() {
    let out_dir = Path::new("icons");
    let icon_path = out_dir.join("icon.png");
    if !icon_path.exists() {
        fs::create_dir_all(out_dir).unwrap();
        fs::write(&icon_path, ICON_PNG).unwrap();
    }
    tauri_build::build();
}

const ICON_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1,
    0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 10, 73, 68, 65, 84,
    120, 156, 99, 0, 1, 0, 0, 5, 0, 1, 13, 10, 45, 180, 0, 0, 0, 0, 73, 69,
    78, 68, 174, 66, 96, 130,
];
