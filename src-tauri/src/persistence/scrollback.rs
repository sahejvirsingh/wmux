use std::io::{Read, Write};
use std::path::PathBuf;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use parking_lot::Mutex;
use rusqlite::Connection;

pub struct ScrollbackDb {
    conn: Mutex<Connection>,
}

impl ScrollbackDb {
    pub fn open(path: PathBuf) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("open db failed: {e}"))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS scrollback (
                pty_id TEXT PRIMARY KEY,
                lines BLOB NOT NULL,
                saved_at INTEGER NOT NULL
            );",
        )
        .map_err(|e| format!("init db failed: {e}"))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn save(&self, pty_id: &str, lines: &[String]) -> Result<(), String> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        let joined = lines.join("\n");
        encoder
            .write_all(joined.as_bytes())
            .map_err(|e| format!("compress failed: {e}"))?;
        let compressed = encoder
            .finish()
            .map_err(|e| format!("compress failed: {e}"))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let conn = self.conn.lock();
        conn.execute(
            "INSERT OR REPLACE INTO scrollback (pty_id, lines, saved_at) VALUES (?1, ?2, ?3)",
            rusqlite::params![pty_id, compressed, now],
        )
        .map_err(|e| format!("save scrollback failed: {e}"))?;
        Ok(())
    }

    pub fn load(&self, pty_id: &str) -> Result<Option<Vec<String>>, String> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT lines FROM scrollback WHERE pty_id = ?1")
            .map_err(|e| format!("prepare failed: {e}"))?;
        let mut rows = stmt
            .query(rusqlite::params![pty_id])
            .map_err(|e| format!("query failed: {e}"))?;
        let Some(row) = rows
            .next()
            .map_err(|e| format!("next failed: {e}"))?
        else {
            return Ok(None);
        };
        let compressed: Vec<u8> = row.get(0).map_err(|e| format!("get failed: {e}"))?;
        let mut decoder = ZlibDecoder::new(compressed.as_slice());
        let mut text = String::new();
        decoder
            .read_to_string(&mut text)
            .map_err(|e| format!("decompress failed: {e}"))?;
        let lines: Vec<String> = text.split('\n').map(|s| s.to_string()).collect();
        Ok(Some(lines))
    }

    pub fn delete(&self, pty_id: &str) -> Result<(), String> {
        let conn = self.conn.lock();
        conn.execute(
            "DELETE FROM scrollback WHERE pty_id = ?1",
            rusqlite::params![pty_id],
        )
        .map_err(|e| format!("delete failed: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_scrollback() {
        let dir = tempfile::tempdir().unwrap();
        let db = ScrollbackDb::open(dir.path().join("test.db")).unwrap();
        let lines: Vec<String> = vec!["line one".into(), "line two".into(), "line three".into()];
        db.save("pty-1", &lines).unwrap();
        let loaded = db.load("pty-1").unwrap().unwrap();
        assert_eq!(loaded, lines);
        assert!(db.load("pty-missing").unwrap().is_none());
    }
}