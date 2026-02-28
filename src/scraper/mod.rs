#[cfg(feature = "scraper")]
use iced::task::{Never, Sipper, sipper};
#[cfg(feature = "scraper")]
use tokio::sync::mpsc;
#[cfg(feature = "scraper")]
use tracing::{debug, error, info, warn};
#[cfg(feature = "scraper")]
use tokio::time::{sleep, Duration};
#[cfg(feature = "scraper")]
use scraper::Selector;
#[cfg(feature = "scraper")]
use reqwest::ClientBuilder;
#[cfg(feature = "scraper")]
use std::{fmt::Display, sync::Arc};
#[cfg(feature = "scraper")]
use tokio::sync::mpsc::{Sender, Receiver};

#[cfg(feature = "scraper")]
use crate::error::ReaderResult;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/143.0.0.0 Safari/537.36";

pub mod db;

#[cfg(feature = "scraper")]
#[derive(Clone)]
struct Extractor {
    name: String,
    link_e: Arc<dyn LinkExtractor>,
    text_e: Arc<dyn TextExtractor>,

    chapters: Vec<String>,
}

#[cfg(feature = "scraper")]
impl Extractor {
    fn chapter_urls(&mut self) -> ReaderResult<()>{
        self.chapters = self.link_e.get_chapter_urls()?;
        Ok(())
    }

    async fn chapter(&self, ch: usize) -> ReaderResult<(String,String)> {
        let url = self.chapters[ch].clone();
        let client = ClientBuilder::new().user_agent(USER_AGENT).build()?;
        let res = client.get(url).send().await?;
        let doc = res.text().await?;
        let header = self.text_e.get_title(doc.as_str())?;
        let text = self.text_e.get_paragraphs(doc.as_str())?;
        Ok((header,text))
    }

    async fn scrap_text(&self, chapter: &str) -> ReaderResult<(String,String)> {
        let client = ClientBuilder::new().user_agent(USER_AGENT).build()?;
        let res = client.get(chapter).send().await?;
        let doc = res.text().await?;
        let header = self.text_e.get_title(doc.as_str())?;
        let text = self.text_e.get_paragraphs(doc.as_str())?;
        if header.to_lowercase().contains("you are human") || header.to_lowercase().contains("are not a robot") {
            return Err(crate::error::ReaderError::Scraper("Maximum number of scraper errors exceeded".to_string()));
        }
        Ok((header,text))
    }

    pub fn count(&self) -> usize {
        self.chapters.len()    
    }

    async fn scrap_book<'b>(&mut self,
        interval: u64,
        output: &mut sipper::Sender<ScraperEvent>, //Sender<ScraperEvent<'b>>, 
        command: &mut Receiver<ScraperCommand>) -> ReaderResult<()> {

        let mut t_interval = interval;
        let mut error_count: u16 = 0;
        let t_conn = db::init_db(self.name.as_str())?;
        let mut i = 1.0;

        self.chapter_urls()?;
        let n = self.chapters.len() as f32;

        for chapter in &self.chapters {
            tokio::select! {
                cmd = { command.recv() } => {
                    match cmd {
                        Some(ScraperCommand::AdjustInterval(i)) => t_interval = i,
                        Some(ScraperCommand::Stop) => {
                            info!("Cancelled scraper!");
                            return Ok(());
                        },
                        None => warn!("None command!"),
                        _ => warn!("Unexpected command!"),
                    }
                },
                v = self.scrap_text(chapter) => {
                    info!("Scraping: {}", chapter);
                    match v {
                        Ok((header,text)) => {
                            sleep(Duration::from_millis(t_interval)).await;
                            match t_conn.execute("INSERT INTO Documents (Title, Content, Line, Character) VALUES (?1,?2,0,0)", (header,text)) {
                                Ok(e) => {
                                    debug!("Inserted text {}: {}", i, e);
                                    output.send(ScraperEvent::Progress( (i,n) )).await;
                                }
                                Err(e) => {
                                    error_count = error_count + 1;
                                    output.send(ScraperEvent::Error(e.to_string())).await;
                                    error!("Error processing text[{}]: {}", i, e);
                                    if error_count > 3 {
                                        return Err(crate::error::ReaderError::Scraper("Maximum number of scraper errors exceeded".to_string()));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error_count = error_count + 1;
                            output.send(ScraperEvent::Error(e.to_string())).await;
                            error!("Error processing text: {}", e);
                            if error_count > 3 {
                                return Err(crate::error::ReaderError::Scraper("Maximum number of scraper errors exceeded".to_string()));
                            }
                        }
                    }
                }
            }
            i = i + 1.0;
        }
        drop(t_conn);
        output.send(ScraperEvent::Finished).await;
        Ok(())
    }

}

//-----[ Link extractor ]-----

#[cfg(feature = "scraper")]
pub trait LinkExtractor: Send + Sync + Display {
    fn get_chapter_urls(&self) -> ReaderResult<Vec<String>>;
}

#[cfg(feature = "scraper")]
#[async_trait::async_trait]
pub trait TextExtractor: Send + Sync + Display {
    fn get_title(&self, doc: &str) -> ReaderResult<String>;
    fn get_paragraphs(&self, doc: &str) -> ReaderResult<String>;
}

#[cfg(feature = "scraper")]
#[derive(Clone)]
struct MainPageExtractor {
    pub main_page_url: String,
    pub pattern: String,
    doc: scraper::Html,
    name: String,
}

#[cfg(feature = "scraper")]
impl LinkExtractor for MainPageExtractor {
    fn get_chapter_urls(&self) -> ReaderResult<Vec<String>> {
        let selector = Selector::parse(self.pattern.as_str())?;
        let links = self.doc.select(&selector)
            .filter_map(|a| a.value().attr("href").map(|x| {
                match x.get(..4) {
                    Some("http") => x.to_string(),
                    Some(_) => {
                        if self.main_page_url.contains("ctext.org") {
                            format!("https://ctext.org/{}", x)
                        } else {
                            format!("{}/{}", self.main_page_url, x)
                        }
                    },
                    None => x.to_string(),
                }
            }))
        .collect::<Vec<_>>();
        Ok(links)
    }
}

#[cfg(feature = "scraper")]
unsafe impl Send for MainPageExtractor {}
#[cfg(feature = "scraper")]
unsafe impl Sync for MainPageExtractor {}

#[cfg(feature = "scraper")]
impl Display for MainPageExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Main page selector: {}", self.name)
    }
}


#[cfg(feature = "scraper")]
#[derive(Clone)]
struct PatternExtractor {
    pub pattern: String,
    pub n_chapters: usize,
    pub name: String,
}

#[cfg(feature = "scraper")]
impl LinkExtractor for PatternExtractor {
    fn get_chapter_urls(&self) -> ReaderResult<Vec<String>> {
        let links = (1..=self.n_chapters)
            .map(|c| format!("{}", self.pattern.replace("{}",c.to_string().as_str())) )
            .collect();
        Ok(links)
    }
}

#[cfg(feature = "scraper")]
impl Display for PatternExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Url pattern extractor")
    }
}

#[cfg(feature = "scraper")]
struct PatternTextExtractor {
    title_pattern: Option<String>,
    pub pattern: String,
    name: String,
}

#[cfg(feature = "scraper")]
impl Display for PatternTextExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PTE[{}]: selector {}", self.name, self.pattern)
    }
}

#[cfg(feature = "scraper")]
#[async_trait::async_trait]
impl TextExtractor for PatternTextExtractor {
    fn get_paragraphs(&self, doc: &str) -> ReaderResult<String> {
        let page = scraper::Html::parse_document(&doc);
        let text = page.select(&Selector::parse(self.pattern.as_str()).unwrap())
            .map(|x| x.text().collect::<String>())
            .reduce(|acc, e| format!("{}\n{}",acc,e.trim()));
        drop(page);
        Ok(text.unwrap_or_default())
    }

    fn get_title(&self, doc: &str) -> ReaderResult<String> {
        let title_selector = self.title_pattern.clone().unwrap_or(String::from("title"));
        let page = scraper::Html::parse_document(&doc);
        let header = page.select(&Selector::parse(title_selector.as_str()).unwrap())
            .map(|x| x.text().collect::<String>())
            .nth(0);
        Ok(header.unwrap_or_default())
    }
}

#[cfg(feature = "scraper")]
struct CTextExtractor {
}

#[cfg(feature = "scraper")]
impl Display for CTextExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CText")
    }
}

#[cfg(feature = "scraper")]
#[async_trait::async_trait]
impl TextExtractor for CTextExtractor {
    fn get_paragraphs(&self, doc: &str) -> ReaderResult<String> {
        let page = scraper::Html::parse_document(&doc);
        let text = page.select(&Selector::parse("div#content3 tr td.ctext").unwrap())
            .filter_map(|x| {
                let text = x.text().collect::<String>();
                if text.starts_with("<td class=\"ctext\"") 
                    && let Some(r_div) = text.find("</div>") 
                        && let Some(p) = text.find("<p>") {
                            let text: &str = &text[r_div+6..p];
                            debug!("text: {}", text);
                            Some(text.to_string())
                        } else {
                            None
                }
            })
        .reduce(|acc, e| format!("{}\n{}",acc,e.trim()));
        drop(page);
        Ok(text.unwrap_or_default())
    }

    fn get_title(&self, doc: &str) -> ReaderResult<String> {
        let title_selector = String::from("title");
        let page = scraper::Html::parse_document(&doc);
        let header = page.select(&Selector::parse(title_selector.as_str()).unwrap())
            .map(|x| x.text().collect::<String>())
            .nth(0);
        Ok(header.unwrap_or_default())
    }
}


#[cfg(feature = "scraper")]
#[derive(Debug,Clone)]
pub enum ScraperEvent {
    Ready(Sender<ScraperCommand>),
    Progress((f32,f32)),
    Finished,
    Error(String),
}


#[cfg(feature = "scraper")]
#[derive(Clone)]
pub enum ScraperCommand {
    Start{ name: String, interval: u64, l_ext: LinkExtractorType, t_ext: TextExtractorType },
    AdjustInterval(u64),
    Stop,
}

#[cfg(feature = "scraper")]
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,PartialEq)]
#[serde(tag = "type", content = "args")]
pub enum LinkExtractorType {
    PatternExtractor{ pattern: String, n_chapters: usize, name: String },
    MainPageExtractor{ url: String, pattern: String, name: String },
}

#[cfg(feature = "scraper")]
impl LinkExtractorType {
    pub const ALL: &'static [Self] = &[
        LinkExtractorType::PatternExtractor { pattern: String::new(), n_chapters: 0, name: String::new() },
        LinkExtractorType::MainPageExtractor { url: String::new(), pattern: String::new(), name: String::new() },
    ];

    pub async fn create(&self) -> ReaderResult<Arc<dyn LinkExtractor>> {
        match &self {
            LinkExtractorType::PatternExtractor { pattern, n_chapters, name } 
            => Ok(Arc::new(PatternExtractor { pattern: pattern.clone(), n_chapters: *n_chapters, name: name.clone() })),
            LinkExtractorType::MainPageExtractor { url, pattern, name } => {
                let client = ClientBuilder::new().user_agent(USER_AGENT).build()?;
                let doc = client.get(url).send().await?;
                let text = doc.text().await?;
                Ok(Arc::new(MainPageExtractor{
                    main_page_url: url.clone(),
                    doc: scraper::Html::parse_document(text.as_str()),
                    pattern: pattern.clone(),
                    name: name.clone(),
                }))
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self {
            LinkExtractorType::PatternExtractor { pattern, n_chapters: _, name } => pattern.is_empty() || name.is_empty(),
            LinkExtractorType::MainPageExtractor { url, pattern, name } => url.is_empty() || pattern.is_empty() || name.is_empty(),
        }
    }
}

#[cfg(feature = "scraper")]
impl Display for LinkExtractorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkExtractorType::PatternExtractor { pattern: _, n_chapters: _, name } => write!(f, "Links [{}]", name),
            LinkExtractorType::MainPageExtractor { url: _, pattern: _, name  } => write!(f, "MainPage [{}]", name),
        }
    }
}

#[cfg(feature = "scraper")]
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize,PartialEq)]
#[serde(tag = "type", content = "args")]
pub enum TextExtractorType {
    PatternTextExtractor{ title_pattern: Option<String>, pattern: String, name: String },
    CText,
}

#[cfg(feature = "scraper")]
impl TextExtractorType {
    pub const ALL: &'static [Self] = &[TextExtractorType::PatternTextExtractor { title_pattern: None, pattern: String::new(), name: String::new() }, TextExtractorType::CText ];

    pub async fn create(&self) -> ReaderResult<Arc<dyn TextExtractor>> {
        match self {
            TextExtractorType::PatternTextExtractor { title_pattern, pattern, name } 
            => Ok(Arc::new(PatternTextExtractor { title_pattern: title_pattern.clone(), pattern: pattern.clone(), name: name.clone() })),
            TextExtractorType::CText
                => Ok(Arc::new( CTextExtractor { } ))
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            TextExtractorType::PatternTextExtractor { title_pattern: _, pattern, name } => pattern.is_empty() || name.is_empty(),
            TextExtractorType::CText => false,
        }
    }
}

#[cfg(feature = "scraper")]
impl Display for TextExtractorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextExtractorType::PatternTextExtractor { title_pattern: _, pattern: _, name } => write!(f, "Text/Pattern [{}]", name),
            TextExtractorType::CText => write!(f, "CText"),
        }
    }
}


#[cfg(feature = "scraper")]
pub fn connect() -> impl Sipper<Never, ScraperEvent> {
    sipper(async |mut output| {
        let (sender, mut receiver) = mpsc::channel::<ScraperCommand>(100);
        output.send(ScraperEvent::Ready(sender)).await;
        loop {
            match receiver.recv().await {
                Some(ScraperCommand::Start { name, interval, l_ext, t_ext }) => {
                    debug!("Start scraper: {}", name);
                    let link_e = l_ext.create().await;
                    let text_e = t_ext.create().await;
                    if link_e.is_ok() && text_e.is_ok() {
                        let mut extractor = Extractor {
                            name,
                            text_e: text_e.unwrap(),
                            link_e: link_e.unwrap(),
                            chapters: vec![],
                        };
                        if let Err(e) = extractor.scrap_book(interval, &mut output, &mut receiver).await {
                            error!("Error extracting book: {}", e);
                            output.send(ScraperEvent::Error("Error extracting book".to_string())).await;
                        }

                    } else {
                        let _ = link_e.inspect_err(|e| {
                            error!("Error creating link extractor: {}", e);
                        });
                        let _ = text_e.inspect_err(|e| {
                            error!("Error creating text extractor: {}", e);
                        });
                        output.send(ScraperEvent::Error("Error creating extractor".to_string())).await;
                    }
                }
                _ => debug!("Got something I should have not"),
            }
        }
    })
}
