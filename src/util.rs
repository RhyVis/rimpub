use std::path::PathBuf;

pub fn get_dir_store() -> PathBuf {
    let path = dirs::home_dir()
        .expect("Failed to get home directory")
        .join(".rimpub");
    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Failed to create directory for rimpub");
    }
    path
}
