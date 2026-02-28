use printpdf::*;
use rusqlite::Connection;
use crate::textbase::{Document, Note, get_notes};
use crate::error::ReaderResult;

static BLACK: printpdf::Op = Op::SetFillColor { col: Color::Rgb(Rgb { r: 0.0, g: 0.0, b: 0.0, icc_profile: None }) };
const PAGE_WIDTH: f64 = 595.0;   // A4 width in points (8.27 in * 72)
const PAGE_HEIGHT: f64 = 842.0;  // A4 height in points (11.69 in * 72)
const MARGIN: f64 = 50.0;

pub fn get_pdf(conn: &Connection, doc_id: u32) -> ReaderResult<Vec<u8>> {
    let mut stmt = conn.prepare("SELECT Title,Content FROM Documents WHERE Id = ?1")?;
    let document = stmt.query_row([doc_id], |row| Ok(
            Document {
                id: doc_id,
                title: row.get(0)?,
                content: row.get(1)?,
                line: 0,
                character: 0,
            }))?;

    let notes = get_notes(conn, doc_id)?;

    let mut pdf = PdfDocument::new(document.title.as_str());
    pdf.metadata.info.creation_date = DateTime::now();
    
    

    Ok(vec![])
}

pub fn get_notes_pdf(conn: &Connection, doc_id: u32) -> ReaderResult<Vec<u8>> {
    let mut stmt = conn.prepare("SELECT Id,Line,Character,Content from Notes WHERE Document = ?1")?;
    let notes = stmt.query_map([doc_id], |row| Ok(
            Note {
                id: row.get(0)?,
                line: row.get(1)?,
                char: row.get(2)?,
                doc: doc_id,
                text: row.get(3)?,
            }))?.map(|e| {
            
        })
        .collect::<Vec<_>>();

    let mut pdf = PdfDocument::new("Notes");
    
    pdf.metadata.info.creation_date = DateTime::now();
    Ok(vec![])
}
