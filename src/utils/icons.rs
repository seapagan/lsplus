use std::fs;

pub fn get_item_icon(metadata: &fs::Metadata) -> String {
    if metadata.is_dir() {
        "\u{f07c}".to_string()
    } else if metadata.is_symlink() {
        "\u{f1177}".to_string()
    } else {
        "".to_string()
    }
}
