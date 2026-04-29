use std::path::PathBuf;

use iced::task::{sipper, Never, Sipper};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};
use tokio::time::{sleep, Duration};
use crate::{config::LlamaType, error::ReaderResult};
use sysinfo::System;
use tokio::process::{Child, Command};
use which::which;

#[derive(Clone,Debug)]
pub enum LlamaCommand {
    ReadyRemote { url: String },
    ReadyLocal { 
        model_path: String, 
        cache_type_k: Option<crate::config::CacheType>, 
        cache_type_v: Option<crate::config::CacheType>, 
        flash_attn: Option<bool>, ctx_size: Option<u32>, 
        n_cpu_moe: Option<u16>, 
        reasoning_budget: Option<u32>, 
        port: Option<u16>,
        presence_penalty: Option<f32>,
        mmproj: Option<String>,
    },
    Start,
    Stop,
    DoNothing,
}

impl From<LlamaType> for LlamaCommand {
    fn from(value: LlamaType) -> Self {
        match value {
            LlamaType::Local { 
                cache_type_k, cache_type_v, flash_attn, ctx_size, n_cpu_moe, 
                reasoning_budget, port, presence_penalty, mmproj 
            } => LlamaCommand::ReadyLocal { model_path: String::new(), cache_type_k, cache_type_v, flash_attn, ctx_size, n_cpu_moe, reasoning_budget, port, mmproj,
                presence_penalty: presence_penalty.map(|c| c.replace(".","").parse::<f32>().ok()).flatten() },
            LlamaType::Remote => LlamaCommand::ReadyRemote { url: String::from("http://localhost:8080") },
            _ => LlamaCommand::DoNothing,
        }
    }
}

impl LlamaCommand {
    pub fn model_path(self, path: &str) -> Self {
        match self {
            LlamaCommand::ReadyLocal { 
                model_path: _, cache_type_k, cache_type_v, 
                flash_attn, ctx_size, n_cpu_moe, reasoning_budget, port, presence_penalty, mmproj 
            } => LlamaCommand::ReadyLocal { model_path: path.to_string(), cache_type_k, cache_type_v, flash_attn, ctx_size, n_cpu_moe, reasoning_budget, port, presence_penalty, mmproj },
            _ => self,
        }
    }
    pub fn url(self, r_url: &str) -> Self {
        match self {
            LlamaCommand::ReadyRemote { .. } => LlamaCommand::ReadyRemote { url: r_url.to_string() },
            _ => self,
        }
    }
}


#[derive(Clone,Debug)]
pub enum LlamaEvent {
    SipperStarted(mpsc::Sender<LlamaCommand>),
    NoLocalFound,
    LocalFoundNotResponding,
    LocalFoundNotRunning,
    LocalFoundRunning,
    Running,
    Error(String),
}

pub fn connect() -> impl Sipper<Never, LlamaEvent> {
    sipper(async |mut output| {
        let (sender, mut receiver) = mpsc::channel::<LlamaCommand>(10);
        let mut ltype: Option<LlamaType> = None;
        let mut laddr = String::from("127.0.0.1:8080");
        let mut child: Option<Child> = None;
        let llama_bin: Option<PathBuf> = which("llama-server").ok();
        let mut path = String::new();
        let client = reqwest::Client::new();

        output.send(LlamaEvent::SipperStarted(sender)).await;
        loop {
            match receiver.try_recv() {
                Ok(LlamaCommand::ReadyLocal { 
                    model_path, 
                    cache_type_k, 
                    cache_type_v, 
                    flash_attn, 
                    ctx_size, 
                    n_cpu_moe, 
                    reasoning_budget, 
                    port,
                    presence_penalty,
                    mmproj}) => {
                    ltype = Some(LlamaType::Local { cache_type_k, cache_type_v, flash_attn, ctx_size, n_cpu_moe, reasoning_budget, port, presence_penalty: presence_penalty.map(|c| c.to_string()), mmproj });
                    laddr = format!("http://127.0.0.1:{}", port.unwrap_or(8080));
                    if model_path.is_empty() {
                        warn!("No model path!");
                    }
                    debug!("llama::sipper::model_path: {}", model_path);
                    path = model_path;
                                    }
                Ok(LlamaCommand::Start) => {
                    if path.is_empty() || !std::path::Path::new(&path).exists() {
                        output.send(LlamaEvent::Error(String::from("Cannot start llama.cpp without valid model path, check AI settings"))).await;
                        continue;
                    }
                    match child.as_mut() {
                        Some(child) => {
                            if let Err(e) = child.kill().await {
                                output.send(LlamaEvent::Error(e.to_string())).await;
                            }
                            info!("Killed stuck process");
                            if let Some(llama_bin) = &llama_bin {
                                match start_llama(llama_bin, ltype.as_ref().unwrap_or(&LlamaType::None), &path).await {
                                    Ok(Some(ch)) => *child = ch,
                                    Ok(None) => warn!("llama started but no handle returned!"),
                                    Err(e) => output.send(LlamaEvent::Error(e.to_string())).await,
                                }
                            }
                        }
                        None => {
                            info!("Starting llama.cpp");
                            if let Some(llama_bin) = &llama_bin {
                                match start_llama(llama_bin, ltype.as_ref().unwrap_or(&LlamaType::None), &path).await {
                                    Ok(ch) => child = ch,
                                    Err(e) => output.send(LlamaEvent::Error(e.to_string())).await,
                                }
                            }
                        }
                    }
                }
                Ok(LlamaCommand::ReadyRemote { url }) => {
                    debug!("Set to remote: {}", url);
                    laddr = url;
                }
                Ok(LlamaCommand::Stop) => {
                    if let Some(child) = child.as_mut() {
                        info!("Stopping");
                        if let Err(e) = child.kill().await {
                            output.send(LlamaEvent::Error(e.to_string())).await;
                        }
                    }
                }
                Ok(LlamaCommand::DoNothing) => {
                    debug!("llama_sipper: do nothing");
                }
                Err(_e) => {
                    let is_running = child.as_ref().is_some_and(|x| x.id().is_some());
                    let is_healthy = is_llama_server_healthy(&client, &laddr).await;
                    match (is_running, is_healthy) {
                        (true,true) => output.send(LlamaEvent::Running).await,
                        (false,true) => output.send(LlamaEvent::LocalFoundRunning).await,
                        (true,false) => output.send(LlamaEvent::LocalFoundNotResponding).await,
                        (false,_) => output.send(LlamaEvent::LocalFoundNotRunning).await,
                    }

                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    })
}

/// Checks if llama-server is actually responding on the /health endpoint
async fn is_llama_server_healthy(client: &reqwest::Client, url: &str) -> bool {
    let url = format!("{}/health",url);
    debug!("llama health check url={}", url);
    let res = client.get(&url)
        .timeout(Duration::from_millis(100))
        .send()
        .await;
    if let Err(e) = &res {
        debug!("Error llama: {}", e);
    }
    
    res.ok().map(|resp| resp.status().is_success())
        .unwrap_or(false)
}

/// Finds PIDs of running `llama-server` processes
fn find_llama_server_pids() -> bool {
    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    sys.processes()
        .values()
        .find(|p| p.name().to_string_lossy().contains("llama-server"))
        .is_some()
}

async fn start_llama(llama_bin: &PathBuf, ltype: &LlamaType, model_path: &str) -> ReaderResult<Option<Child>> {
    match ltype {
        LlamaType::Local { 
            cache_type_k, 
            cache_type_v, 
            flash_attn, 
            ctx_size, 
            n_cpu_moe, 
            reasoning_budget, 
            port,
            presence_penalty,
            mmproj} => {
            let mut cmd = Command::new(llama_bin);
            if let Some(cache_type_k) = cache_type_k {
                cmd.arg("--cache-type-k").arg(cache_type_k.to_string());
            }
            if let Some(cache_type_v) = cache_type_v {
                cmd.arg("--cache-type-v").arg(cache_type_v.to_string());
            }
            match flash_attn {
                Some(true) => { cmd.arg("--flash-attn").arg("on"); },
                Some(false) =>{ cmd.arg("--flash-attn").arg("off"); },
                None => {},
            }
            cmd.arg("-m").arg(&model_path);
            if let Some(ctx_size) = ctx_size {
                cmd.arg("--ctx-size").arg(ctx_size.to_string());
            }
            if let Some(n_cpu_moe) = n_cpu_moe {
                cmd.arg("--n-cpu-moe").arg(n_cpu_moe.to_string());
            }
            if let Some(reasoning_budget) = reasoning_budget {
                cmd.arg("--reasoning-budget").arg(reasoning_budget.to_string());
            }
            if let Some(presence_penalty) = presence_penalty {
                cmd.arg("--presence-penalty").arg(presence_penalty.to_string());
            }
            if let Some(mmproj) = mmproj {
                cmd.arg("--mmproj").arg(mmproj.to_string());
            }
            cmd.arg("--port").arg(port.unwrap_or(8080).to_string());

            Ok(Some(cmd.spawn()?))
        }
        _ => Ok(None),
    }
}
