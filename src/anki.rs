use std::collections::HashSet;
use chrono::{DateTime, Days, Utc};
use rusqlite::Connection;
use std::fmt;

use crate::error::ReaderResult;

#[derive(Debug,Clone,PartialEq)]
pub struct AnkiEntry {
    word: String,
    deck: i64,
    added: DateTime<Utc>,
}

impl AnkiEntry {
    pub fn new(word: &str, did: i64, id: i64) -> Self {
        let added = DateTime::from_timestamp_millis(id)
            .unwrap_or_else(|| Utc::now());

        Self {
            word: word.to_string(),
            added,
            deck: did,
        }
    }
}

impl fmt::Display for AnkiEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.word, self.added)
    }
}

pub fn last_n_days(conn: &Connection, n: u64) -> ReaderResult<Vec<AnkiEntry>> {
    let d_from = Utc::now()
        .checked_sub_days(Days::new(n))
        .unwrap();
    let d_from_timestamp = d_from.timestamp_millis();
    let mut st = conn.prepare("SELECT id,did,REPLACE(sfld, CHAR(10), ' ') FROM notes WHERE id > ?")?;
    let rows = st.query_map([d_from_timestamp], |row| {
        let word: String = row.get(2)?;
        Ok( AnkiEntry::new( word.as_str(), row.get(1)?, row.get(0)? ) )
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn search_anki(conn: &Connection, sel: &str) -> ReaderResult<Vec<String>> {
    let mut st = conn.prepare("SELECT REPLACE(sfld, CHAR(10), ' ') FROM notes WHERE REPLACE(sfld, CHAR(10), '') LIKE ?")?;
    let pattern = format!("%{}%", sel);
    let rows = st.query_map([pattern], |row| row.get(0))?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn count_anki(conn: &Connection) -> ReaderResult<usize> {
    let res = conn.query_row("SELECT COUNT(*) FROM notes",
        [], |row| row.get(0) )?;
    Ok(res)
}

pub fn anki_chars(conn: &Connection) -> ReaderResult<HashSet<char>> {
    let mut st = conn.prepare("SELECT REPLACE(sfld, CHAR(10), ' ') FROM notes")?;
    let rows = st.query_map([], |row| row.get(0) )?;
    let mut results = HashSet::<char>::new();
    for row in rows {
        let r: String = row?;
        r.chars().for_each(|c| { results.insert(c); } );
    }
    Ok(results)
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_search() {
        let conn = Connection::open("/home/morti/.local/share/Anki2/User 1/collection.anki2").unwrap();
        let r = search_anki(&conn, "年").unwrap();
        assert!(r.len() > 0);
    }

    #[test]
    fn test_chars() {
        let conn = Connection::open("/home/morti/.local/share/Anki2/User 1/collection.anki2").unwrap();
        let r = anki_chars(&conn).unwrap();
        let cs = r.len();
        assert!(cs > 3000);
    }
}
