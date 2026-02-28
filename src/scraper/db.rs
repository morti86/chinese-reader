use rusqlite::Connection;
use crate::error::ReaderResult;

pub fn init_db(file_name: impl AsRef<std::path::Path>) -> ReaderResult<Connection> {
    let conn = Connection::open(file_name)?;

    let exists: i32 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
        &[&"Documents"],
        |row| row.get(0)
    )?;

    if exists == 0 {
        conn.execute(
            "CREATE TABLE Documents (
                Id INTEGER, 
                Title TEXT NOT NULL,
                Content BLOB,
                Line INTEGER DEFAULT 0,
                Character INTEGER DEFAULT 0,
                PRIMARY KEY(Id AUTOINCREMENT) )", 
            ())?;

        conn.execute("
            CREATE TABLE Notes (
                Id INTEGER,
                Line INTEGER NOT NULL,
                Character INTEGER,
                Document INTEGER NOT NULL,
                Content TEXT,
                PRIMARY KEY(Id AUTOINCREMENT) )", 
            ())?;
    }
    Ok(conn)
}

