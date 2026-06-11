use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipEntry {
    pub id: i64,
    pub content: String,
    pub content_hash: String,
    pub content_type: String,
    pub source_app: Option<String>,
    pub is_pinned: bool,
    pub tags: Option<String>,
    pub created_at: String,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&app_dir)?;
        let db_path = app_dir.join("clipvault.db");

        let conn = Connection::open(db_path)?;
        let db = Database {
            conn: Mutex::new(conn),
        };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS clips (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                content     TEXT NOT NULL,
                content_hash TEXT NOT NULL UNIQUE,
                content_type TEXT NOT NULL DEFAULT 'text',
                source_app  TEXT,
                is_pinned   INTEGER NOT NULL DEFAULT 0,
                tags        TEXT,
                created_at  TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_clips_created_at ON clips(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_clips_pinned ON clips(is_pinned);
            CREATE INDEX IF NOT EXISTS idx_clips_hash ON clips(content_hash);",
        )?;
        Ok(())
    }

    pub fn insert(&self, entry: &ClipEntry) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO clips (content, content_hash, content_type, source_app, is_pinned, tags, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(content_hash) DO UPDATE SET created_at = excluded.created_at",
            params![
                entry.content,
                entry.content_hash,
                entry.content_type,
                entry.source_app,
                entry.is_pinned as i32,
                entry.tags,
                entry.created_at,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
        pinned_only: bool,
    ) -> Result<Vec<ClipEntry>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = String::from(
            "SELECT id, content, content_hash, content_type, source_app, is_pinned, tags, created_at
             FROM clips WHERE 1=1",
        );

        if pinned_only {
            sql.push_str(" AND is_pinned = 1");
        }

        if !query.is_empty() {
            sql.push_str(" AND content LIKE ?1");
            sql.push_str(" ORDER BY is_pinned DESC, created_at DESC LIMIT ?2 OFFSET ?3");

            let pattern = format!("%{}%", query);
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![pattern, limit as i64, offset as i64], |row| {
                Ok(ClipEntry {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    content_hash: row.get(2)?,
                    content_type: row.get(3)?,
                    source_app: row.get(4)?,
                    is_pinned: row.get(5)?,
                    tags: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
        } else {
            sql.push_str(" ORDER BY is_pinned DESC, created_at DESC LIMIT ?1 OFFSET ?2");

            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
                Ok(ClipEntry {
                    id: row.get(0)?,
                    content: row.get(1)?,
                    content_hash: row.get(2)?,
                    content_type: row.get(3)?,
                    source_app: row.get(4)?,
                    is_pinned: row.get(5)?,
                    tags: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
        }
    }

    pub fn toggle_pin(&self, id: i64) -> Result<bool, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE clips SET is_pinned = CASE WHEN is_pinned = 0 THEN 1 ELSE 0 END WHERE id = ?1",
            params![id],
        )?;

        let pinned: bool = conn.query_row(
            "SELECT is_pinned FROM clips WHERE id = ?1",
            params![id],
            |row| row.get::<_, i32>(0),
        )? == 1;

        Ok(pinned)
    }

    pub fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM clips WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn cleanup(&self, keep_count: usize) -> Result<usize, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM clips WHERE is_pinned = 0 AND id NOT IN (
                SELECT id FROM clips WHERE is_pinned = 0
                ORDER BY created_at DESC LIMIT ?1
            )",
            params![keep_count as i64],
        )?;
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup_db() -> Database {
        // Each test gets a unique directory, cleaned before use
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("clipvault_test_{}", n));
        let _ = std::fs::remove_dir_all(&dir); // clean previous run's data
        Database::new(dir).expect("failed to create test db")
    }

    fn make_entry(content: &str, time: &str) -> ClipEntry {
        ClipEntry {
            id: 0,
            content: content.to_string(),
            content_hash: format!("hash_{}", content),
            content_type: "text".to_string(),
            source_app: None,
            is_pinned: false,
            tags: None,
            created_at: time.to_string(),
        }
    }

    #[test]
    fn test_insert_and_search() {
        let db = setup_db();
        let e1 = make_entry("hello world", "2026-01-01T00:00:00Z");
        let e2 = make_entry("rust tauri", "2026-01-02T00:00:00Z");

        db.insert(&e1).unwrap();
        db.insert(&e2).unwrap();

        let results = db.search("hello", 10, 0, false).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "hello world");
    }

    #[test]
    fn test_deduplicate_by_hash() {
        let db = setup_db();
        let e1 = make_entry("same content", "2026-01-01T00:00:00Z");
        let e2 = ClipEntry {
            content: "same content".to_string(),
            content_hash: e1.content_hash.clone(),
            ..e1.clone()
        };

        let id1 = db.insert(&e1).unwrap();
        let id2 = db.insert(&e2).unwrap();

        // Same hash → same row, timestamp updated
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_toggle_pin() {
        let db = setup_db();
        let entry = make_entry("pin me", "2026-01-01T00:00:00Z");
        let id = db.insert(&entry).unwrap();

        assert!(!entry.is_pinned);

        let now_pinned = db.toggle_pin(id).unwrap();
        assert!(now_pinned);

        let now_unpinned = db.toggle_pin(id).unwrap();
        assert!(!now_unpinned);
    }

    #[test]
    fn test_delete() {
        let db = setup_db();
        let entry = make_entry("delete me", "2026-01-01T00:00:00Z");
        let id = db.insert(&entry).unwrap();

        db.delete(id).unwrap();

        let results = db.search("delete", 10, 0, false).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_cleanup() {
        let db = setup_db();
        for i in 0..5 {
            let entry = make_entry(&format!("clip {}", i), &format!("2026-01-0{}T00:00:00Z", i + 1));
            db.insert(&entry).unwrap();
        }

        let deleted = db.cleanup(2).unwrap();
        assert_eq!(deleted, 3); // 5 total, keep 2 → 3 deleted
    }

    #[test]
    fn test_pinned_only() {
        let db = setup_db();
        let mut e1 = make_entry("important", "2026-01-01T00:00:00Z");
        e1.is_pinned = true;
        let e2 = make_entry("normal", "2026-01-02T00:00:00Z");

        db.insert(&e1).unwrap();
        db.insert(&e2).unwrap();

        let results = db.search("", 10, 0, true).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "important");
    }
}
