use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

/// A simple pipeline for generating text with the OpenAI API.
///
/// # Examples
///
/// ```no_run
/// use talea::openai_pipeline::OpenAiPipeline;
/// let pipeline = OpenAiPipeline::new("MY_KEY", "gpt-3.5-turbo");
/// let text = pipeline.ask("Say hello").unwrap();
/// println!("{}", text);
/// ```
pub struct OpenAiPipeline {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiPipeline {
    /// Create a new pipeline with the provided API key and model name.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        OpenAiPipeline {
            client: Client::new(),
            api_key: api_key.into(),
            model: model.into(),
        }
    }

    /// Create a pipeline using the `OPENAI_API_KEY` environment variable.
    pub fn from_env(model: impl Into<String>) -> Result<Self, String> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| "Missing OPENAI_API_KEY environment variable".to_string())?;
        Ok(Self::new(api_key, model))
    }

    /// Change the model used by the pipeline.
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = model.into();
    }

    /// Change the API key used by the pipeline.
    pub fn set_api_key(&mut self, api_key: impl Into<String>) {
        self.api_key = api_key.into();
    }

    /// Send a prompt to the OpenAI API and return the generated text.
    pub fn generate(&self, prompt: &str) -> Result<String, String> {
        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let res: ChatResponse = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .map_err(|e| e.to_string())?
            .json()
            .map_err(|e| e.to_string())?;

        res.choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| "No content in response".to_string())
    }

    /// Ask the model a question in plain English. This is an alias for [`generate`]
    /// but reads more naturally in code.
    pub fn ask(&self, prompt: &str) -> Result<String, String> {
        self.generate(prompt)
    }

    /// Convenience helper that reads the API key from the environment,
    /// uses `gpt-3.5-turbo`, sends the prompt, and writes the response to
    /// a file. This allows running the pipeline in a single line.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use talea::openai_pipeline::OpenAiPipeline;
    /// OpenAiPipeline::quick(
    ///     "Explain the significance of the Renaissance.",
    ///     "renaissance.txt",
    /// ).unwrap();
    /// ```
    pub fn quick(prompt: &str, output_path: impl AsRef<std::path::Path>) -> Result<(), String> {
        let pipeline = Self::from_env("gpt-3.5-turbo")?;
        let text = pipeline.generate(prompt)?;
        std::fs::write(output_path, text).map_err(|e| e.to_string())
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::OpenAiPipeline;

    #[test]
    fn builds_from_env_without_key() {
        std::env::remove_var("OPENAI_API_KEY");
        let result = OpenAiPipeline::from_env("gpt-3.5-turbo");
        assert!(result.is_err());
    }
}
