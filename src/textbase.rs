use rusqlite::{Connection, OptionalExtension, params};
use tracing::{debug, info, error};
use crate::error::ReaderResult;
use std::fmt::{Display, Result as FResult};
use rig::Embed;
use tokio::io::{BufReader, AsyncBufReadExt};

#[derive(Embed, Clone, Default, Debug)]
pub struct Document {
    pub id: u32,
    pub title: String,
    #[embed]
    pub content: String,
    pub line: usize,
    pub character: usize,
}

impl Document {
    pub fn is_empty(&self) -> bool {
        self.id == 0
    }

    pub fn title(&self) -> String {
        format!("{} | {},{}", self.title, self.line, self.character)
    }
}

impl PartialEq for Document {
    fn eq(&self, d: &Document) -> bool {
        self.id == d.id
    }
}

impl Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> FResult {
        f.write_str(&self.title.as_str())
    }
}

#[derive(Clone, Default, Debug)]
pub struct Note {
    pub id: u32,
    pub doc: u32,
    pub line: usize,
    pub char: usize,
    pub text: String,
}

impl Note {
    pub fn is_empty(&self) -> bool {
        self.id == 0 || self.doc == 0
    }

    pub fn pos(&self) -> (usize,usize) {
        (self.line,self.char)
    }
}

impl PartialEq for Note {
    fn eq(&self, d: &Note) -> bool {
        self.id == d.id || (self.doc == d.doc && self.line == d.line && self.char == d.char)
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> FResult {
        f.write_str(&self.text.as_str())
    }
}

pub fn get_documents(conn: &Connection) -> ReaderResult<Vec<Document>> {
    let mut stmt = conn.prepare("SELECT Id, Title, Line, Character FROM Documents")?;
    let doc_iter = stmt.query_map([], |row| {
        Ok(Document {
            id: row.get(0)?,
            title: row.get(1)?,
            content: String::new(),
            line: row.get(2)?,
            character: row.get(3)?,
        })
    })?;

    Ok(doc_iter.map(|doc| doc.unwrap()).collect())
}

pub fn update_progress(conn: &mut Connection, id: usize, character: usize, line: usize) -> ReaderResult<()> {
    let tx = conn.transaction()?;
    tx.execute("UPDATE Documents SET Character = ?1, Line = ?2 WHERE Id = ?3", [character, line, id])?;
    tx.commit()?;
    Ok(())
}

pub fn get_content(conn: &Connection, doc_id: u32) -> ReaderResult<Option<String>> {
    let mut stmt = conn.prepare("SELECT Content FROM Documents WHERE Id = ?1")?;
    let result = stmt.query_row([doc_id], |row| row.get(0)).optional()?;
    Ok(result)
}

pub fn save_text(conn: &mut Connection, id: u32, title: &str, content: &str) -> ReaderResult<i64> {
    debug!("Save text {id}/{title}");
    let tx = conn.transaction()?;
    if id == 0 {
        debug!("INSERT title={}", title);
        tx.execute("INSERT INTO Documents (Title, Content, Line) VALUES (?1, ?2, 0)", [title, content])?;
    } else {
        debug!("UPDATE title={}", title);
        tx.execute("UPDATE Documents SET Content = ?2, title = ?3 WHERE Id = ?1", params![id, content, title])?;
    }
    let id = tx.last_insert_rowid();
    tx.commit()?;
    Ok(id)
}

pub fn save_note(conn: &mut Connection, line: usize, character: usize, document: u32, content: &str) -> ReaderResult<i64> {
    debug!("Save note: {}/{}:{}",document,line,character);
    let tx = conn.transaction()?;
    tx.execute("INSERT INTO Notes (Line, Character, Document, Content) VALUES (?1, ?2, ?3, ?4)", params![line, character, document, content])?;
    let id = tx.last_insert_rowid();
    tx.commit()?;
    Ok(id)
}

pub fn delete_text(conn: &mut Connection, title: &str) -> ReaderResult<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM Notes WHERE Document IN (SELECT Id FROM Documents WHERE Title = ?1)", [title])?;
    tx.execute("DELETE FROM Documents WHERE Title = ?1", [title])?;
    tx.commit()?;
    Ok(())
}

pub fn delete_note(conn: &mut Connection, document: u32, line: usize, character: usize) -> ReaderResult<()> {
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM Notes WHERE Document = ?1 AND Line = ?2 AND Character = ?3", params![document, line, character])?;
    tx.commit()?;
    Ok(())
}

pub fn get_notes(conn: &Connection, document: u32) -> ReaderResult<Vec<Note>> {
    let mut stmt = conn.prepare("SELECT Id, Line, Character, Document, Content FROM Notes WHERE Document = ?1 ORDER BY Line,Character")?;
    let notes_iter = stmt.query_map([document], |row| {
        Ok(Note {
            id: row.get(0)?,
            line: row.get(1)?,
            char: row.get(2)?,
            doc: row.get(3)?,
            text: row.get(4)?,
        })
    })?;

    Ok(notes_iter.map(|doc| doc.unwrap()).collect())
}

pub fn export_notes(conn: &Connection, document: u32) -> ReaderResult<String> {
    let notes = get_notes(conn, document)?
        .iter().map(|note| note.to_string())
        .reduce(|acc,c| format!("{}\n{}",acc,c));
    
    Ok(notes.unwrap_or_default())
}

pub async fn get_doc_md(db_file: &str, doc_id: u32) -> ReaderResult<String> {
    let db = db_file.to_string();
    let cc: ReaderResult<(Option<String>, Vec<Note>)> = tokio::task::spawn_blocking(move || {
        let conn = Connection::open(db)?;
        let c = get_content(&conn, doc_id)?;
        let n = get_notes(&conn, doc_id)?;
        Ok((c,n))
    }).await?;
    match cc {
        Ok((None,_)) => {
            info!("None found for document!");
            Ok(String::new())
        },
        Ok((Some(doc),notes)) => {
            debug!("Found doc!");
            let reader = BufReader::new(std::io::Cursor::new(doc.as_str()));
            let mut lines = reader.lines();
            
            let mut result = String::new();
            let link_len = "[*](n:{}:{})".len();
            result.reserve(link_len * notes.len() + doc.len());
            let mut last_pos: usize;
            let mut i: usize = 0;

            while let Ok(Some(line)) = lines.next_line().await {
                debug!("New line found: {}", i);
                last_pos = 0;

                let line_notes = notes.iter().filter(|n| n.line == i);

                for note in line_notes {
                    debug!("Inserting string at {}:{}, last_pos={}", note.line, note.char, last_pos);
                    result.push_str(&line[last_pos..note.char]);
                    result.push_str( format!("[*](n:{}:{})",note.line,note.char).as_str() );
                    last_pos = note.char;
                };
                result.push_str(&line[last_pos..]);
                result.push_str("\n\n");
                debug!("Finished line {}", i);
                i = i + 1;
            }
            debug!("Finished: {}", result.len());
            Ok(result)
        },
        Err(e) => {
            error!("Error loading md: {}", e);
            Err(e)
        },
    }
}
