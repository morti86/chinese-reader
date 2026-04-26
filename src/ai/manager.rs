use std::marker::PhantomData;

use rig::{agent::Agent, client::{CompletionClient, ModelListingClient}, completion::Prompt, providers::{
    anthropic, deepseek, gemini, mistral, ollama, openai::{self, OpenAICompletionsExt}, xai
    }, streaming::{StreamedAssistantContent, StreamingChat, StreamingPrompt}, wasm_compat::WasmCompatSend
};
use sipper::{Sender, StreamExt};

use crate::config::Provider;
use crate::error::ReaderResult;

use super::ChatEvent;

pub trait AiManagerState {}
pub struct Init;
impl AiManagerState for Init {}
pub struct Ready;
impl AiManagerState for Ready {}

pub type AiChat = AiManager<Ready>;


macro_rules! stream_prompt {
    ($t:expr, $p:expr, $o:expr, $m:ty) => {
        {
            let mut r = $t.as_ref().unwrap().stream_prompt($p).await;
            while let Some(r) = r.next().await {
                match r {
                    Ok(chunk) => {
                        tracing::debug!("Received message");
                        match chunk {
                            rig::agent::MultiTurnStreamItem::<$m>::StreamAssistantItem(StreamedAssistantContent::Text(text)) => {
                                tracing::debug!("text=[{:?}]", text);
                                $o.send(ChatEvent::Text(text.text)).await;
                            },
                            rig::agent::MultiTurnStreamItem::<$m>::StreamAssistantItem(StreamedAssistantContent::ToolCall{ tool_call, .. }) => {
                                tracing::debug!("tool call id={}, function={:?}", tool_call.id, tool_call.function);
                            }
                            rig::agent::MultiTurnStreamItem::<$m>::StreamAssistantItem(StreamedAssistantContent::ToolCallDelta{ content, .. }) => {
                                tracing::debug!("tool call delta: {:?}", content);
                            }
                            rig::agent::MultiTurnStreamItem::<$m>::StreamAssistantItem(StreamedAssistantContent::Reasoning(re)) => {
                                tracing::debug!("Reasoning: {:?}", re);
                            }
                            rig::agent::MultiTurnStreamItem::<$m>::StreamAssistantItem(StreamedAssistantContent::ReasoningDelta{ reasoning, .. }) => {
                                tracing::debug!("Reasoning delta: {:?}", reasoning);
                            }
                            _ => $o.send(ChatEvent::Final).await,
                        }
                    }
                    Err(e) => {
                        tracing::error!("Streaming error: {}", e);
                        $o.send(ChatEvent::ChatError(e.to_string())).await;
                    }
                }
            }
        }
    };
    ($t:expr, $p:expr, $o:expr, $m:ty, $hist:expr) => {
        {
            let mut r = $t.as_ref().unwrap().stream_chat($p,$hist).await;
            while let Some(r) = r.next().await {
                match r {
                    Ok(chunk) => {
                        tracing::debug!("Received message");
                        match chunk {
                            rig::agent::MultiTurnStreamItem::<$m>::StreamAssistantItem(StreamedAssistantContent::Text(text)) => {
                                $o.send(ChatEvent::Text(text.text)).await;
                            },
                            rig::agent::MultiTurnStreamItem::<$m>::StreamAssistantItem(StreamedAssistantContent::ToolCall{ tool_call, .. }) => {
                                tracing::debug!("tool call id={}, function={:?}", tool_call.id, tool_call.function);
                            }
                            rig::agent::MultiTurnStreamItem::<$m>::StreamAssistantItem(StreamedAssistantContent::ToolCallDelta{ content, .. }) => {
                                tracing::debug!("tool call delta: {:?}", content);
                            }
                            rig::agent::MultiTurnStreamItem::<$m>::StreamAssistantItem(StreamedAssistantContent::Reasoning(re)) => {
                                tracing::debug!("Reasoning: {:?}", re);
                            }
                            rig::agent::MultiTurnStreamItem::<$m>::StreamAssistantItem(StreamedAssistantContent::ReasoningDelta{ reasoning, .. }) => {
                                tracing::debug!("Reasoning delta: {:?}", reasoning);
                                $o.send(ChatEvent::ReasoningDelta(reasoning)).await;
                            }
                            _ => $o.send(ChatEvent::Final).await,
                        }
                    }
                    Err(e) => {
                        tracing::error!("Streaming error: {}", e);
                        $o.send(ChatEvent::ChatError(e.to_string())).await;
                    }
                }
            }
        }
    };

}

#[derive(Clone)]
pub enum AiManager<S: AiManagerState> {
    Openai { 
        client: openai::Client, 
        agent: Option<Agent<rig::providers::openai::responses_api::ResponsesCompletionModel>>, 
        preamble: Option<String>,
        _pd: PhantomData<S> 
    },
    Deepseek { 
        client: deepseek::Client,
        agent: Option<Agent<deepseek::CompletionModel>>,
        preamble: Option<String>,
    },
    Anthropic { 
        client: anthropic::Client,
        agent: Option<Agent<anthropic::completion::CompletionModel>>,
        preamble: Option<String>,
    },
    Ollama { 
        client: ollama::Client,
        agent: Option<Agent<ollama::CompletionModel>>,
        preamble: Option<String>,
    },
    Xai { 
        client: xai::Client,
        agent: Option<Agent<xai::CompletionModel>>,
        preamble: Option<String>,
    },
    Gemini { 
        client: gemini::Client,
        agent: Option<Agent<gemini::CompletionModel>>,
        preamble: Option<String>,
    },
    Mistral { 
        client: mistral::Client,
        agent: Option<Agent<mistral::CompletionModel>>,
        preamble: Option<String>,
    },
    LlamaCpp {
        client: rig::client::Client<OpenAICompletionsExt>,
        agent: Option<Agent<rig::providers::openai::GenericCompletionModel<OpenAICompletionsExt>>>, 
        preamble: Option<String>,
    }
}

impl AiManager<Init> {
    pub fn new(provider: Provider, api_key: &str) -> crate::error::ReaderResult<AiManager<Init>> {
        let s = match provider {
            Provider::Openai => AiManager::<Init>::Openai { client: openai::Client::new(api_key)?, agent: None, preamble: None, _pd: PhantomData },
            Provider::Xai => AiManager::<Init>::Xai { client: xai::Client::new(api_key)?, agent: None, preamble: None },
            Provider::Ollama => AiManager::<Init>::Ollama { client: ollama::Client::new(rig::client::Nothing)?, agent: None, preamble: None },
            Provider::Gemini => AiManager::<Init>::Gemini { client: gemini::Client::new(api_key)?, agent: None, preamble: None },
            Provider::Mistral => AiManager::<Init>::Mistral { client: mistral::Client::new(api_key)?, agent: None, preamble: None },
            Provider::Deepseek => AiManager::<Init>::Deepseek { client: deepseek::Client::new(api_key)?, agent: None, preamble: None },
            Provider::Anthropic => AiManager::<Init>::Anthropic { client: anthropic::Client::new(api_key)?, agent: None, preamble: None },
            Provider::LlamaCpp => AiManager::<Init>::LlamaCpp { client: openai::Client::builder().base_url("http://localhost:8080").api_key(api_key).build()?.completions_api(), agent: None, preamble: None },
        };
        Ok(s)
    }

    pub fn new_ollama_url(url: &str) -> crate::error::ReaderResult<AiManager<Init>> {
        Ok(AiManager::<Init>::Ollama { 
            client: ollama::Client::builder().base_url(url).api_key(rig::client::Nothing).build()?,
            agent: None,
            preamble: None,
        })
    }

    pub fn new_llama_cpp_url(url: &str, key: &str) -> crate::error::ReaderResult<AiManager<Init>> {
        Ok(AiManager::<Init>::LlamaCpp { 
            client: openai::Client::builder().base_url(url).api_key(key).build()?.completions_api(),
            agent: None,
            preamble: None,
        } )
    }

    pub fn preamble(self, preamble: &str) -> AiManager<Init> {
        let preamble = Some(preamble.to_string());
        match self {
            Self::Openai { client, agent, preamble: _, _pd } => {
                AiManager::<Init>::Openai { client, agent, preamble,_pd: PhantomData }
            }
            Self::Ollama { client, agent, preamble: _ } => {
                AiManager::<Init>::Ollama { client, agent, preamble }
            }
            Self::Deepseek { client, agent, preamble: _, } => {
                AiManager::<Init>::Deepseek { client, agent, preamble }
            }
            Self::Anthropic { client, agent, preamble: _, } => {
                AiManager::<Init>::Anthropic { client, agent, preamble }
            }
            Self::Xai { client, agent, preamble: _, } => {
                AiManager::<Init>::Xai { client, agent, preamble }
            }
            Self::Gemini { client, agent, preamble: _, } => {
                AiManager::<Init>::Gemini { client, agent, preamble }
            }
            Self::Mistral { client, agent, preamble: _, } => {
                AiManager::<Init>::Mistral { client, agent, preamble }
            }
            Self::LlamaCpp { client, agent, preamble } => {
                AiManager::<Init>::LlamaCpp { client, agent, preamble }
            }
        }
    }


    pub async fn list_models(&self) -> ReaderResult<Vec<String>> {
        match self {
            Self::Openai { client, .. } => {
                let models = client.list_models().await?;
                Ok(models.data.iter().map(|m| m.display_name().to_string()).collect())
            }
            Self::Deepseek { client: _, agent: _, preamble: _, } => {
                let models = crate::utils::get_models(&Provider::Deepseek);
                Ok(models)
            }
            Self::Ollama { client, ..} => {
                let models = client.list_models().await?;
                Ok(models.data.iter().map(|m| m.display_name().to_string()).collect())
            }
            Self::Anthropic { client: _, agent: _, preamble: _, } => {
                let models = crate::utils::get_models(&Provider::Anthropic);
                Ok(models)
            }
            Self::Xai { client: _, agent: _, preamble: _, } => {
                let models = crate::utils::get_models(&Provider::Xai);
                Ok(models)
            }
            Self::Gemini { client, agent: _, preamble: _, } => {
                let models = client.list_models().await?;
                Ok(models.data.iter().map(|m| m.display_name().to_string()).collect())
            }
            Self::Mistral { client, .. } => {
                let models = client.list_models().await?;
                Ok(models.data.iter().map(|m| m.display_name().to_string()).collect())
            }
            Self::LlamaCpp { .. } => {
                Ok(vec![])
            }


        }
    }

    pub fn ready(self, model: &str) -> AiManager<Ready> {
        tracing::debug!("Setting up model: {}", model);
        match self {
            Self::Openai { client, preamble, .. } => {
                let r = client.agent(model).preamble(&preamble.clone().unwrap_or_default()).build();
                AiManager::<Ready>::Openai { client, agent: Some(r), preamble, _pd: PhantomData }
            }
            Self::Ollama { client, preamble, .. } => {
                let mut r = client.agent(model).preamble(&preamble.clone().unwrap_or_default()).build();
                r.max_tokens = Some(8192);
                AiManager::<Ready>::Ollama { client, agent: Some(r), preamble }
            }
            Self::Deepseek { client, agent: _, preamble, } => {
                let r = client.agent(model).preamble(&preamble.clone().unwrap_or_default()).build();
                AiManager::<Ready>::Deepseek { client, agent: Some(r), preamble }
            }
            Self::Anthropic { client, agent: _, preamble, } => {
                let r = client.agent(model).preamble(&preamble.clone().unwrap_or_default()).build();
                AiManager::<Ready>::Anthropic { client, agent: Some(r), preamble }
            }
            Self::Xai { client, agent: _, preamble, } => {
                let r = client.agent(model).preamble(&preamble.clone().unwrap_or_default()).build();
                AiManager::<Ready>::Xai { client, agent: Some(r), preamble }
            }
            Self::Gemini { client, agent: _, preamble, } => {
                let r = client.agent(model).preamble(&preamble.clone().unwrap_or_default()).build();
                AiManager::<Ready>::Gemini { client, agent: Some(r), preamble }
            }
            Self::Mistral { client, agent: _, preamble} => {
                let r = client.agent(model).preamble(&preamble.clone().unwrap_or_default()).build();
                AiManager::<Ready>::Mistral { client, agent: Some(r), preamble }
            }
            Self::LlamaCpp { client, preamble, .. } => {
                let r = client.agent(model).preamble(&preamble.clone().unwrap_or_default()).build();
                AiManager::<Ready>::LlamaCpp { client, agent: Some(r), preamble }
            }

        }
    }
}

impl AiManager<Ready> {
    pub fn is_local_llama(&self) -> bool {
        match self {
            Self::Openai { .. } => true,
            _ => false,
        }
    }

    pub async fn prompt(&self, prompt: impl Into<rig::completion::Message> + WasmCompatSend) -> ReaderResult<String> {
        match self {
            Self::Xai { agent, ..} => {
                Ok(agent.as_ref().unwrap().prompt(prompt).await?)
            }
            Self::Openai { agent, ..} => {
                Ok(agent.as_ref().unwrap().prompt(prompt).await?)
            }
            Self::Ollama { agent, ..} => {
                Ok(agent.as_ref().unwrap().prompt(prompt).await?)
            }
            Self::Gemini { client: _, agent, preamble: _ } => {
                Ok(agent.as_ref().unwrap().prompt(prompt).await?)
            }
            Self::Mistral { client: _, agent, .. } => {
                Ok(agent.as_ref().unwrap().prompt(prompt).await?)
            }
            Self::Deepseek { client: _, agent, preamble: _ } => {
                Ok(agent.as_ref().unwrap().prompt(prompt).await?)
            }
            Self::Anthropic { client: _, agent, preamble: _ } => {
                Ok(agent.as_ref().unwrap().prompt(prompt).await?)
            }
            Self::LlamaCpp { agent, .. } => {
                Ok(agent.as_ref().unwrap().prompt(prompt).await?)
            }
        }
    }

    pub async fn stream_prompt(&self, prompt: impl Into<rig::completion::Message> + WasmCompatSend, output: &mut Sender<ChatEvent>) {
        match self {
            Self::Xai { client: _, agent, preamble: _ } => {
                stream_prompt!(agent, prompt, output, rig::providers::openai::responses_api::streaming::StreamingCompletionResponse);
            }
            Self::Openai { agent, ..} => {
                stream_prompt!(agent, prompt, output, rig::providers::openai::responses_api::streaming::StreamingCompletionResponse);
            }
            Self::Ollama { agent, ..} => {
                stream_prompt!(agent, prompt, output, rig::providers::ollama::StreamingCompletionResponse);
            }
            Self::Gemini { client: _, agent, preamble: _ } => {
                stream_prompt!(agent, prompt, output, rig::providers::gemini::streaming::StreamingCompletionResponse);
            }
            Self::Mistral { agent, .. } => {
                stream_prompt!(agent, prompt, output, rig::providers::mistral::completion::CompletionResponse);
            }
            Self::Deepseek { client: _, agent, preamble: _ } => {
                stream_prompt!(agent, prompt, output, rig::providers::deepseek::StreamingCompletionResponse);
            }
            Self::Anthropic { client: _, agent, preamble: _ } => {
                stream_prompt!(agent, prompt, output, rig::providers::anthropic::streaming::StreamingCompletionResponse);
            }
            Self::LlamaCpp { agent, .. } => {
                stream_prompt!(agent, prompt, output, rig::providers::openai::StreamingCompletionResponse);
            }
        }
    }

    pub async fn stream_chat(&self, prompt: impl Into<rig::completion::Message> + WasmCompatSend, output: &mut Sender<ChatEvent>, chat_history: Vec<rig::message::Message>) {
        match self {
            Self::Xai { client: _, agent, preamble: _ } => {
                stream_prompt!(agent, prompt, output, rig::providers::openai::responses_api::streaming::StreamingCompletionResponse, chat_history);
            }
            Self::Openai { agent, ..} => {
                stream_prompt!(agent, prompt, output, rig::providers::openai::responses_api::streaming::StreamingCompletionResponse, chat_history);
            }
            Self::Ollama { agent, ..} => {
                stream_prompt!(agent, prompt, output, rig::providers::ollama::StreamingCompletionResponse, chat_history);
            }
            Self::Gemini { client: _, agent, preamble: _ } => {
                stream_prompt!(agent, prompt, output, rig::providers::gemini::streaming::StreamingCompletionResponse, chat_history);
            }
            Self::Mistral { agent, .. } => {
                stream_prompt!(agent, prompt, output, rig::providers::mistral::completion::CompletionResponse, chat_history);
            }
            Self::Deepseek { client: _, agent, preamble: _ } => {
                stream_prompt!(agent, prompt, output, rig::providers::deepseek::StreamingCompletionResponse, chat_history);
            }
            Self::Anthropic { client: _, agent, preamble: _ } => {
                stream_prompt!(agent, prompt, output, rig::providers::anthropic::streaming::StreamingCompletionResponse, chat_history);
            }
            Self::LlamaCpp { agent, ..} => {
                stream_prompt!(agent, prompt, output, rig::providers::openai::StreamingCompletionResponse, chat_history);
            }

        }
    }
}
