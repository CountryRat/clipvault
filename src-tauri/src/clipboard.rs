use arboard::Clipboard;
use chrono::Utc;

use crate::db::ClipEntry;

/// Hash the content for deduplication
fn hash_content(content: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(content.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Try to read text from clipboard and return a ClipEntry if there's new content
pub fn try_read_clipboard(last_hash: &mut Option<String>) -> Option<ClipEntry> {
    let mut clipboard = match Clipboard::new() {
        Ok(c) => c,
        Err(_) => return None,
    };

    // Get text content
    let content = match clipboard.get_text() {
        Ok(text) => text,
        Err(_) => return None,
    };

    // Skip empty content
    if content.trim().is_empty() {
        return None;
    }

    let hash = hash_content(&content);

    // Skip if same as last read (prevents duplicates during polling)
    if let Some(ref last) = last_hash {
        if *last == hash {
            return None;
        }
    }

    *last_hash = Some(hash.clone());

    Some(ClipEntry {
        id: 0, // auto-assigned by DB
        content,
        content_hash: hash,
        content_type: "text".to_string(),
        source_app: None,
        is_pinned: false,
        tags: None,
        created_at: Utc::now().to_rfc3339(),
    })
}
