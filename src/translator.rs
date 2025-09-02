use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

pub struct Translator {
    client: Client,
    config: Config,
}

impl Translator {
    pub fn new(config: &Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout()))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config: config.clone(),
        }
    }

    pub async fn translate_line<F>(
        &self,
        text: &str,
        target_lang: &str,
        source_lang: Option<&str>,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(&str, &str),
    {
        if text.is_empty() {
            callback(text, "");
            return Ok(());
        }

        let max_retries = 3;
        let mut last_error = None;

        for attempt in 1..=max_retries {
            match self
                .translate_line_attempt(text, target_lang, source_lang)
                .await
            {
                Ok(result) => {
                    callback(text, &result);
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        let delay = Duration::from_millis(1000 * attempt);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    async fn translate_line_attempt(
        &self,
        text: &str,
        target_lang: &str,
        source_lang: Option<&str>,
    ) -> Result<String> {
        let prompt = self.build_line_translation_prompt(text, target_lang, source_lang);

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a professional translator. Translate the given text accurately while preserving the original meaning and tone. Only return the translated text without any explanations or additional content. If the input contains no valid characters or is empty, return an empty line.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let request = ChatRequest {
            model: self.config.model().to_string(),
            messages,
            temperature: self.config.temperature(),
            max_tokens: self.config.max_tokens(),
        };

        let url = format!("{}/chat/completions", self.config.endpoint());
        let mut req_builder = self.client.post(&url).json(&request);

        // Add authorization header if API key is available
        if let Some(api_key) = self.config.api_key() {
            if !api_key.is_empty() {
                req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
            }
        }

        let response = req_builder
            .send()
            .await
            .context("Failed to send translation request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse API response")?;

        if chat_response.choices.is_empty() {
            return Err(anyhow!("No translation choices returned from API"));
        }

        let translated_text = chat_response.choices[0].message.content.trim();

        // Remove quotes if the response is wrapped in them
        let cleaned_text = if (translated_text.starts_with('"') && translated_text.ends_with('"'))
            || (translated_text.starts_with('\'') && translated_text.ends_with('\''))
        {
            &translated_text[1..translated_text.len() - 1]
        } else {
            translated_text
        };

        Ok(cleaned_text.to_string())
    }

    pub async fn translate_word<F>(
        &self,
        word: &str,
        target_lang: &str,
        source_lang: Option<&str>,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(&str, &str),
    {
        let max_retries = 3;
        let mut last_error = None;

        for attempt in 1..=max_retries {
            match self
                .translate_word_attempt(word, target_lang, source_lang)
                .await
            {
                Ok(translation) => {
                    callback(word, &translation);
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        let delay = Duration::from_millis(1000 * attempt);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    async fn translate_word_attempt(
        &self,
        word: &str,
        target_lang: &str,
        source_lang: Option<&str>,
    ) -> Result<String> {
        let prompt = self.build_word_translation_prompt(word, target_lang, source_lang);

        let messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a professional translator and dictionary. When given a single word or phrase, provide the most common translation with brief context if needed. Only return the translation without explanations. If the input contains no valid characters or is empty, return an empty line.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let request = ChatRequest {
            model: self.config.model().to_string(),
            messages,
            temperature: self.config.temperature(),
            max_tokens: self.config.max_tokens(),
        };

        let url = format!("{}/chat/completions", self.config.endpoint());
        let mut req_builder = self.client.post(&url).json(&request);

        // Add authorization header if API key is available
        if let Some(api_key) = self.config.api_key() {
            if !api_key.is_empty() {
                req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
            }
        }

        let response = req_builder
            .send()
            .await
            .context("Failed to send word translation request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "API request failed with status {}: {}",
                status,
                error_text
            ));
        }

        let chat_response: ChatResponse = response
            .json()
            .await
            .context("Failed to parse API response")?;

        if chat_response.choices.is_empty() {
            return Err(anyhow!("No translation choices returned from API"));
        }

        let translated_text = chat_response.choices[0].message.content.trim();

        // Remove quotes if the response is wrapped in them
        let cleaned_text = if (translated_text.starts_with('"') && translated_text.ends_with('"'))
            || (translated_text.starts_with('\'') && translated_text.ends_with('\''))
        {
            &translated_text[1..translated_text.len() - 1]
        } else {
            translated_text
        };

        Ok(cleaned_text.to_string())
    }

    fn build_line_translation_prompt(
        &self,
        text: &str,
        target_lang: &str,
        source_lang: Option<&str>,
    ) -> String {
        let target_lang_name = self.lang_code_to_name(target_lang);

        match source_lang {
            Some(source) => {
                let source_lang_name = self.lang_code_to_name(source);
                format!(
                    "Translate the following text from {} to {}:\n\n{}",
                    source_lang_name, target_lang_name, text
                )
            }
            None => {
                format!(
                    "Translate the following text to {}:\n\n{}",
                    target_lang_name, text
                )
            }
        }
    }

    fn build_word_translation_prompt(
        &self,
        word: &str,
        target_lang: &str,
        source_lang: Option<&str>,
    ) -> String {
        let target_lang_name = self.lang_code_to_name(target_lang);

        match source_lang {
            Some(source) => {
                let source_lang_name = self.lang_code_to_name(source);
                format!(
                    "Translate this {} word to {}: {}",
                    source_lang_name, target_lang_name, word
                )
            }
            None => {
                format!("Translate this word to {}: {}", target_lang_name, word)
            }
        }
    }

    fn lang_code_to_name(&self, code: &str) -> String {
        match code {
            "zh" | "zh-cn" => "Chinese".to_string(),
            "zh-tw" => "Traditional Chinese".to_string(),
            "en" => "English".to_string(),
            "ja" => "Japanese".to_string(),
            "ko" => "Korean".to_string(),
            "fr" => "French".to_string(),
            "de" => "German".to_string(),
            "es" => "Spanish".to_string(),
            "it" => "Italian".to_string(),
            "pt" => "Portuguese".to_string(),
            "ru" => "Russian".to_string(),
            "ar" => "Arabic".to_string(),
            "hi" => "Hindi".to_string(),
            "th" => "Thai".to_string(),
            "vi" => "Vietnamese".to_string(),
            _ => code.to_string(), // fallback to the code itself
        }
    }
}
