use std::fs;
use std::path::Path;

/// Load .env file, setting environment variables that are not already set.
pub fn load_env_file(file_path: Option<&str>) {
    let path = file_path.unwrap_or(".env");
    if !Path::new(path).exists() {
        return;
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Strip BOM
    let content = content.strip_prefix('\u{FEFF}').unwrap_or(&content);

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some(sep_index) = trimmed.find('=') else {
            continue;
        };
        if sep_index == 0 {
            continue;
        }

        let key = trimmed[..sep_index].trim();
        let value = strip_quotes(trimmed[sep_index + 1..].trim());

        if key.is_empty() {
            continue;
        }

        // Only set if not already defined
        if std::env::var(key).is_err() {
            std::env::set_var(key, value);
        }
    }
}

fn strip_quotes(value: &str) -> &str {
    let trimmed = value.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        if trimmed.len() >= 2 {
            return &trimmed[1..trimmed.len() - 1];
        }
    }
    trimmed
}
