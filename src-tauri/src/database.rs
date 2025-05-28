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
                value TEXT,
                source TEXT,
                hash TEXT PRIMARY KEY
            )
        ";

        connection
            .execute(create_table_str, ())
            .expect("Failed to create table");
        println!("Done with table create command");
        Database { conn: connection }
    }
    pub fn upsert(&self, clip: Clip) -> Result<()> {
        let query_string = "
            INSERT INTO history VALUES(:timestamp, :value, :hash, :source)
            ON CONFLICT(hash) DO UPDATE SET timestamp=:timestamp
        ";
        let res = self
            .conn
            .execute(query_string, (clip.timestamp, clip.value, clip.hash, clip.source));
        if res.is_err() {
            println!("Failed to insert value");
        } else {
            println!("Inserted value");
        }
        Ok(())
    }
    pub fn get_all_clips_str(&self, limit: u32) -> Vec<String> {
        self.get_all_clips(limit)
            .unwrap()
            .into_iter()
            .map(|x| x.value)
            .collect()
    }
    pub fn get_all_clips(&self, limit: u32) -> Result<Vec<Clip>> {
        let query_string = format!("
            SELECT timestamp, value, hash, source FROM history ORDER BY TIMESTAMP DESC LIMIT {limit}
        ");
        let mut stmt = self.conn.prepare(query_string.as_str())?;
        let clips_iter = stmt.query_map([], |row| {
            Ok(Clip {
                timestamp: row.get(0)?,
                value: row.get(1)?,
                hash: row.get(2)?,
                source: row.get(3).unwrap_or(String::from("")),
            })
        });
        match clips_iter {
            Ok(clips) => clips.collect(),
            Err(E) => panic!("Error at get clips {}", E)
        }
        // let clips: Vec<Clip> = clips_iter.collect::<Result<Vec<_>>>();
        // Ok(clips)
    }

    pub fn search(&self, pattern: String) -> Result<Vec<Clip>> {
        let query_string = format!("SELECT timestamp, value, hash, source FROM history WHERE value like \"%{}%\" order by timestamp desc", pattern);
        let mut stmt = self.conn.prepare(query_string.as_str())?;
        let clips_iter = stmt.query_map([], |row| {
            Ok(Clip {
                timestamp: row.get(0)?,
                value: row.get(1)?,
                hash: row.get(2)?,
                source: row.get(3).unwrap_or(String::from("")),
            })
        })?;
        let clips: Vec<Clip> = clips_iter.collect::<Result<Vec<_>>>()?;
        Ok(clips)
    }
}
