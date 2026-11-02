use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text(String),
    /// Image data in base64 format, e.g., "data:image/jpeg;base64,/9j/4AAQSkZJRg..." or just the raw base64 string
    Image {
        data: String,
        mime_type: String,
    },
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ImageGenResponse {
    pub images_base64: Vec<String>,
}
