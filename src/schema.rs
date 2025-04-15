use serde::{Deserialize, Serialize};

pub mod account;
pub use account as Account;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllowedImageType { Gif, Png, Jpeg, Webp }