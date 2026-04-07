//! String utility functions.

/// Split a file path into directory and filename components.
/// Returns `(directory, filename)` where directory includes the trailing separator.
pub fn split_path(path: &str) -> (&str, &str) {
    match path.rfind(|c| c == '/' || c == '\\') {
        Some(pos) => (&path[..=pos], &path[pos + 1..]),
        None => ("", path),
    }
}

/// Get the file extension from a path (without the dot), in lowercase.
pub fn get_extension(path: &str) -> Option<&str> {
    let filename = split_path(path).1;
    match filename.rfind('.') {
        Some(pos) if pos < filename.len() - 1 => Some(&filename[pos + 1..]),
        _ => None,
    }
}

/// Convert a string to snake_case.
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 4);
    let mut prev_is_upper = false;
    let mut prev_is_separator = true;

    for (i, ch) in s.chars().enumerate() {
        if ch == ' ' || ch == '-' || ch == '_' {
            if !result.is_empty() && !prev_is_separator {
                result.push('_');
            }
            prev_is_separator = true;
            prev_is_upper = false;
        } else if ch.is_uppercase() {
            if !prev_is_upper && !prev_is_separator && i > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            prev_is_upper = true;
            prev_is_separator = false;
        } else {
            result.push(ch);
            prev_is_upper = false;
            prev_is_separator = false;
        }
    }

    result
}

/// Simple FNV-1a 32-bit hash of a string. Useful for fast lookups.
pub fn hash_string(s: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    for byte in s.bytes() {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_path() {
        assert_eq!(split_path("foo/bar/baz.txt"), ("foo/bar/", "baz.txt"));
        assert_eq!(split_path("baz.txt"), ("", "baz.txt"));
        assert_eq!(split_path("foo\\bar.txt"), ("foo\\", "bar.txt"));
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension("foo.png"), Some("png"));
        assert_eq!(get_extension("path/to/file.WAV"), Some("WAV"));
        assert_eq!(get_extension("noext"), None);
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("CamelCase"), "camel_case");
        assert_eq!(to_snake_case("XMLParser"), "xmlparser");
        assert_eq!(to_snake_case("hello world"), "hello_world");
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_hash_string() {
        let h1 = hash_string("hello");
        let h2 = hash_string("hello");
        let h3 = hash_string("world");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
}
