use deepl::DeepLApi;
use iced::task::{sipper, Never, Sipper};
use rig::{message::Message as Rmsg, streaming::{ToolCallDeltaContent}};
use tokio::sync::{RwLock, mpsc};
use crate::{config::Language, error::{ReaderError, ReaderResult}};
use tracing::{debug, info, warn};
use crate::AGENT_NEW;
pub mod manager;

#[derive(Clone)]
pub enum ChatCommand {
    Request{ message: Rmsg, chat_history: Vec<Rmsg> },
}

#[derive(Debug,Clone)]
pub enum ChatEvent {
    ChatReady{ sender: mpsc::Sender<ChatCommand>, cts: mpsc::Sender<CancellationToken>},
    Text(String),
    ToolCall { id: String, function: String, args: String },
    ToolCallDelta(ToolCallDeltaContent),
    Reasoning(rig::completion::message::Reasoning),
    Final,
    ChatError(String),
}

#[derive(Debug)]
pub struct CancellationToken;

pub fn connect() -> impl Sipper<Never, ChatEvent> {
    sipper(async |mut output| {
        let (sender, mut receiver) = mpsc::channel::<ChatCommand>(10);
        let (cts, mut ctr) = mpsc::channel::<CancellationToken>(10);

        output.send(ChatEvent::ChatReady { sender, cts }).await;
        
        loop {
            tokio::select! {
                v = receiver.recv() => {
                   match v {
                       Some(ChatCommand::Request { message, chat_history }) => {
                           match AGENT_NEW.get() {
                               Some(md) => {
                                   let mdr = md.read().await;
                                   mdr.stream_chat(message, &mut output).await;
                               }
                               None => {
                                    warn!("Ai chat not configured!");
                               }
                           }
                       }
                       None => {
                           debug!("nothing");
                       }
                   }
                }
                _ = ctr.recv() => {
                    info!("Action cancelled!");
                }
            }
        }
    })
}

impl Into<deepl::Lang> for Language {
    fn into(self) -> deepl::Lang {
        match self {
            Language::English => deepl::Lang::EN_US,
            Language::日本語 => deepl::Lang::JA,
            Language::Polski => deepl::Lang::PL,
            Language::Türkçe => deepl::Lang::TR,
            Language::Deutsch => deepl::Lang::DE,
            Language::Русский => deepl::Lang::RU,
            Language::Español => deepl::Lang::ES,
            Language::Italiano => deepl::Lang::IT,
            Language::Française => deepl::Lang::FR,
        }
    }
}

pub async fn ask_deepl(text: &str, lang: Language, key: &str) -> ReaderResult<String> {
    let api = DeepLApi::with(key).new();
    let res = api.translate_text(text, lang.into()).await?;
    Ok(res.to_string())
}
