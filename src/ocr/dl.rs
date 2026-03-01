use iced::{futures::StreamExt, task::{sipper, Never, Sipper}};
use reqwest::Client;
use tokio::sync::mpsc;
use tracing::{error, info};
use std::{fs::File, io::Write};
use crate::error::ReaderError;

#[derive(Clone)]
pub enum DlCommand {
    GetFile { url: String, dest: String },
    Stop
}

#[derive(Clone, Debug)]
pub enum DlEvent {
    Ready(mpsc::Sender<DlCommand>),
    Progress(f32),
    Error(ReaderError),
    End,
}

pub fn connect() -> impl Sipper<Never, DlEvent> {
    sipper(async |mut output| {
        let client = Client::new();
        let (sender, mut receiver) = mpsc::channel::<DlCommand>(100);
        output.send(DlEvent::Ready(sender)).await;

        loop {
            match receiver.recv().await {
                Some(DlCommand::GetFile { url, dest }) => {
                    match File::create(&dest) {
                        Ok(mut file) => {
                            match client.get(url).send().await {
                                Ok(response) => {
                                    let total_size = response.content_length().unwrap_or(0) as f32;
                                    let mut stream = response.bytes_stream();
                                    let mut downloaded = 0.0;
                                    while let Some(chunk) = stream.next().await {
                                        match chunk {
                                            Ok(chunk) => {
                                                let _ = file.write_all(&chunk);
                                                downloaded = downloaded + (chunk.len() as f32 / total_size);
                                                output.send(DlEvent::Progress(downloaded)).await;
                                            }
                                            Err(e) => {
                                                error!("{}",e);
                                            }
                                        }
                                    }
                                    output.send(DlEvent::End).await;
                                }
                                Err(e) => {
                                    error!("{}",e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("{}",e);
                        }
                    }
                }
                Some(DlCommand::Stop) => {
                    info!("Stop");
                    continue;
                }
                None => {
                }
            }
        }
    })
}
