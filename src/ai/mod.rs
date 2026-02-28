use deepl::DeepLApi;
use iced::{futures::StreamExt, task::{sipper, Never, Sipper}};
use rig::{message::Message as Rmsg, streaming::{StreamedAssistantContent, StreamingChat}
};

use tokio::sync::mpsc;
use crate::{config::Language, error::{ReaderError, ReaderResult}, AGENT};
use tracing::{debug, error, warn};

#[derive(Clone)]
pub enum ChatCommand {
    Request{ message: Rmsg, chat_history: Vec<Rmsg> },
    Stop,
}

#[derive(Clone, Debug)]
pub enum ChatEvent {
    ChatReady(mpsc::Sender<ChatCommand>),
    Message(StreamedAssistantContent<()>),
    ChatError(ReaderError),
}

pub fn connect() -> impl Sipper<Never, ChatEvent> {
    sipper(async |mut output| {
        let (sender, mut receiver) = mpsc::channel::<ChatCommand>(100);
        output.send(ChatEvent::ChatReady(sender)).await;

        loop {
            match receiver.recv().await {
                Some(ChatCommand::Request{message, chat_history}) => {
                    match AGENT.get() {
                        None => {
                            error!("Trying to send to no agent");
                            output.send(ChatEvent::ChatError(ReaderError::Ai("Trying to ask a non-existant agent".to_string()))).await;
                        }
                        Some(agent) => {
                            if let Ok(mut stream) = agent.stream_chat(message, chat_history).await {
                                while let Some(r) = stream.next().await {
                                    match r {
                                        Ok(chunk) => {
                                            debug!("Received message");
                                            output.send(ChatEvent::Message(chunk)).await
                                        }
                                        Err(e) => {
                                            error!("Streaming error: {}", e);
                                            output.send(ChatEvent::ChatError(ReaderError::Ai(e.to_string()))).await;
                                        }
                                    }
                                }
                            }
                            
                        }
                    }

                }
                Some(ChatCommand::Stop) => {
                }
                None => {
                    warn!("Received None!!!");
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
