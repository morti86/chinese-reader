use rusqlite::{Connection, Row};
use std::{collections::{HashMap, HashSet}, fmt};
use rayon::prelude::*;
use tracing::{debug, warn};
use std::collections::BTreeMap;
use crate::{anki::AnkiEntry, utils::is_chinese_char};
use crate::error::ReaderResult;

type Dupa<T> = Result<T, Box<dyn std::error::Error>>;
pub const HSK_TOTAL: [f32; 7] = [477.0, 736.0, 940.0, 971.0, 1056.0, 1076.0, 5301.0];

#[derive(Clone, Debug)]
pub struct Entry {
    sim: String,
    tra: String,
    pin: String,
    mea: String,
    hsk: Option<u32>,
    chr: bool,
    idx: char,
    link: Option<String>,
    anki: Option<crate::anki::AnkiEntry>,
}

impl Entry {
    pub fn from_row(r: &Row) -> Self {
        let sim: String = r.get_unwrap(0);
        let chr = sim.chars().count() == 1;
        let idx = sim.chars().nth(0).unwrap_or('?');
        Self {
            sim,
            tra: r.get_unwrap(1),
            pin: r.get_unwrap(2),
            mea: r.get_unwrap(3),
            hsk: r.get_unwrap(4),
            chr,
            idx,
            link: None,
            anki: None,
        }
    }

    pub fn index(&self) -> char {
        self.idx
    }

    pub fn to_md(&self) -> String {
        let meanings = self.mea.split("/")
            .map(|s| {
                if s.starts_with("variant of") {
                    let han: String = s.chars().filter(|c| is_chinese_char(c)).collect();
                    if !han.is_empty() {
                        format!("[variant of {}](c:{})\n",han,han)
                    } else {
                        String::new()
                    }
                } else if s.starts_with("used in") {
                    let sep_ix = s.find("|");
                    let end_ix = s.find("[");
                    if sep_ix.is_some() && end_ix.is_some() {
                        let han = &s[sep_ix.unwrap()+1..end_ix.unwrap()];
                        format!("[used in {}](c:{})\n",han,han)
                    } else {
                        s.to_string()
                    }
                } else {
                    s.to_string()
                }
            })
            .reduce(|acc,a| format!("{}\n- {}",acc,a));

        let hsk = if self.hsk.is_some() { format!("HSK{}", self.hsk.unwrap()) } else { String::new() };
        let anki = if self.anki.is_some() { " (A) " } else { "" };
        match meanings {
            None => format!("Error formatting meanings!"),
            Some(meanings) => format!("\n## {} | {}\n *{}* {}\n {}\n- {}", self.sim, self.tra, self.pin, hsk, anki, meanings )
        }
    }

    pub fn get_variant(&self) -> Option<String> {
        self.mea.split("/")
            .filter_map(|s| {
                if s.starts_with("variant of") {
                    Some(s.chars().filter(|c| is_chinese_char(c)).collect())
                } else {
                    None
                }
            })
            .nth(0)
    }

    pub fn meanings(&self) -> &str {
        self.mea.as_str()
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let hsk = if self.hsk.is_some() { format!("HSK{}", self.hsk.unwrap()) } else { String::new() };
        write!(f, "- {} | {} [{}] {}\n- {}", self.sim, self.tra, self.pin, hsk, self.mea.replace("/","\n- "))
    }
}

pub struct Cedict {
    data_t: BTreeMap<char, Vec<Entry>>,
    data_hsk: HashMap<u32,Vec<Entry>>,
    anki: HashSet<AnkiEntry>,
}

impl Cedict {
    pub fn new(fname: &str, anki_fname: &Option<String>) -> Dupa<Self> {
        let path = format!("/usr/share/cnreader/{}",fname);
        let conn = match std::fs::exists(&path) {
            Ok(true) => {
                debug!("CEDICT found at: {}", path);
                Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?
            }
            _ => {
                debug!("CEDICT fallback: {}", fname);
                Connection::open_with_flags(fname, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?
            }
        };          
        let mut st = conn.prepare("SELECT * from Cedict")?;
        
        let mut data_t: BTreeMap<char, Vec<Entry>> = BTreeMap::new();
        let mut data_hsk: HashMap<u32, Vec<Entry>> = HashMap::new();
        let mut data_tr = st.query([])?;

        debug!("Querying Anki");

        let anki = if let Some(anki_fname) = anki_fname
            && let Ok(true) = std::fs::exists(anki_fname) {
            let anki_conn = Connection::open_with_flags(anki_fname, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY);
            match anki_conn {
                Ok(aconn) => crate::anki::anki_words_entry(&aconn)?,
                Err(e) => {
                    warn!("Error connecting Anki: {}", e);
                    HashSet::new()
                }
            }
        } else {
            warn!("Empty Anki!");
            HashSet::new()
        };

        debug!("Anki base loaded: {}", anki.len());

        while let Ok(next) = data_tr.next() {
            if let Some(row) = next {
                let mut e = Entry::from_row(row);
                e.anki = anki.iter().find(|x| x.word.eq(&e.sim)).cloned();
                let k = e.index();
                if let Some(hsk) = e.hsk {
                    data_t.entry(k).or_default().push(e.clone());
                    data_hsk.entry(hsk).and_modify(|x| x.push(e));
                } else {
                    data_t.entry(k).or_default().push(e);
                }
            } else {
                warn!("Row with problems!");
                break;
            }
        }


        Ok(Self { 
            data_t,
            data_hsk,
            anki,
        })
    }

    pub fn characters(&self) -> Vec<&Entry> {
        self.data_t.par_iter()
            .map(|(_,v)| v.iter().filter(|&e| e.chr).collect())
            .reduce(|| vec![], |a,b| ([a,b]).concat() )
    }

    fn characters_filtered(&self, s: &str) -> Vec<&Entry> {
        self.data_t.par_iter()
            .map(|(_,v)| v.iter().filter(|&e| e.chr && (s.contains(e.sim.as_str()) || s.contains(e.tra.as_str())) ).collect())
            .reduce(|| vec![], |a,b| ([a,b]).concat() )

    }

    /// Convert traditional to simplified
    pub fn to_sim(&self, s: &str) -> String {
        let chr_list = self.characters_filtered(s);
        let ss = s.par_chars()
            .map(|z| {
                let c = chr_list.iter().find(|x| x.tra == z.to_string());
                match c {
                    Some(c) => c.sim.chars().nth(0).unwrap_or(z),
                    None => z,
                }
                
            }).collect();
        ss
    }

    pub fn len(&self) -> usize {
        self.data_t.len()
    }

    /// Search all containing
    pub fn search(&self, s: &str) -> Vec<&Entry> {
        self.data_t.par_iter()
            .map(|(_,v)| v.iter().filter(|&e| e.sim.contains(s)).collect() )
            .reduce(|| vec![], |a,b| ([a,b]).concat() )
    }

    /// Search exact match
    pub fn find(&self, s: &str) -> Vec<&Entry> {
        if s.is_empty() {
            return vec![];
        }
        debug!("find: {}", s);
        let c = s.chars().nth(0).unwrap();
        if let Some(r) = self.data_t.get(&c) {
            return r.iter()
                .filter(|e| e.sim.as_str() == s)
                .collect::<Vec<&Entry>>();
        }
        vec![]
    }

    /// How many entries that are in Anki for each HSK level there are?
    pub fn count_hsk_anki(&self) -> HashMap<u32, usize> {
        self.data_hsk.par_iter()
            .map(|(hsk, e)| (*hsk, e.iter().filter(|x| x.anki.is_some()).count() ) )
            .collect()
    }

    /// How many entries that are for each HSK level there are?
    pub fn count_hsk(&self) -> HashMap<u32, usize> {
        self.data_hsk.par_iter()
            .map(|(hsk, e)| (*hsk, e.len()) )
            .collect()
    }

}


