use crate::clip::Clip;
use dirs;
use rusqlite::{params, Connection, Result};

pub(crate) struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Self {
        let app_data_dir = dirs::data_dir().map(|mut path| {
            path.push("cliphistory");
            path
        });
        println!("app_data_dir: {:?}", app_data_dir);
        let connection = Connection::open(app_data_dir.unwrap()).unwrap();
        let create_table_str: &str = "
            CREATE TABLE IF NOT EXISTS history(
                timestamp LONG DEFAULT(strftime(\'%s\','now') || substr(strftime(\'%f\','now'),4)),
                value TEXT, hash TEXT PRIMARY KEY
            )
        ";
        connection
            .execute(create_table_str, ())
            .expect("Failed to create table");
        Database { conn: connection }
    }
    pub fn upsert(&self, clip: Clip) -> Result<()> {
        let query_string = "
            INSERT INTO history VALUES(:timestamp, :value, :hash)
            ON CONFLICT(hash) DO UPDATE SET timestamp=:timestamp
        ";
        let res = self
            .conn
            .execute(query_string, (clip.timestamp, clip.value, clip.hash));
        if res.is_err() {
            println!("Failed to insert value");
        } else {
            println!("Inserted value");
        }
        Ok(())
    }
    pub fn get_all_clips_str(&self) -> Vec<String> {
        // res
        self.get_all_clips()
            .unwrap()
            .into_iter()
            .map(|x| x.value)
            .collect()
    }
    pub fn get_all_clips(&self) -> Result<Vec<Clip>> {
        let query_string = "
            SELECT timestamp, value, hash FROM history ORDER BY TIMESTAMP DESC
        ";
        let mut stmt = self.conn.prepare(query_string)?;
        let clips_iter = stmt.query_map([], |row| {
            Ok(Clip {
                timestamp: row.get(0)?,
                value: row.get(1)?,
                hash: row.get(2)?,
            })
        })?;
        let clips: Vec<Clip> = clips_iter.collect::<Result<Vec<_>>>()?;
        Ok(clips)
    }

    pub fn search(&self, pattern: String) -> Result<Vec<Clip>> {
        let query_string = format!("SELECT timestamp, value, hash FROM history WHERE value like \"%{}%\" order by timestamp desc", pattern);
        let mut stmt = self.conn.prepare(query_string.as_str())?;
        let clips_iter = stmt.query_map([], |row| {
            Ok(Clip {
                timestamp: row.get(0)?,
                value: row.get(1)?,
                hash: row.get(2)?,
            })
        })?;
        let clips: Vec<Clip> = clips_iter.collect::<Result<Vec<_>>>()?;
        Ok(clips)
    }
}
