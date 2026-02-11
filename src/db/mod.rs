use anyhow::Result;
use rusqlite::{params, Connection};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ChunkState {
    pub download_id: String,
    pub chunk_index: usize,
    pub downloaded: u64,
    pub complete: bool,
}

pub struct ResumeStore {
    conn: Mutex<Connection>,
}

impl ResumeStore {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS chunk_state (
                download_id TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                downloaded INTEGER NOT NULL,
                complete INTEGER NOT NULL,
                PRIMARY KEY (download_id, chunk_index)
            );
            ",
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn save_chunk_state(&self, state: &ChunkState) -> Result<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "
            INSERT INTO chunk_state (download_id, chunk_index, downloaded, complete)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(download_id, chunk_index)
            DO UPDATE SET downloaded=excluded.downloaded, complete=excluded.complete
            ",
            params![
                state.download_id,
                state.chunk_index as i64,
                state.downloaded as i64,
                state.complete as i64
            ],
        )?;
        Ok(())
    }

    pub fn load_chunk_state(&self, download_id: &str) -> Result<Vec<ChunkState>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn.prepare(
            "SELECT download_id, chunk_index, downloaded, complete FROM chunk_state WHERE download_id=?1",
        )?;

        let rows = stmt.query_map([download_id], |row| {
            Ok(ChunkState {
                download_id: row.get(0)?,
                chunk_index: row.get::<_, i64>(1)? as usize,
                downloaded: row.get::<_, i64>(2)? as u64,
                complete: row.get::<_, i64>(3)? == 1,
            })
        })?;

        Ok(rows.filter_map(Result::ok).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persists_and_reads_chunk_state() {
        let db = ResumeStore::new(":memory:").expect("db");
        db.save_chunk_state(&ChunkState {
            download_id: "d1".into(),
            chunk_index: 0,
            downloaded: 42,
            complete: false,
        })
        .expect("save");

        let states = db.load_chunk_state("d1").expect("load");
        assert_eq!(states.len(), 1);
        assert_eq!(states[0].downloaded, 42);
    }
}
