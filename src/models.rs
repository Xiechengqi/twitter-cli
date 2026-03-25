use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub screen_name: String,
    pub name: String,
    pub bio: String,
    pub location: String,
    pub url: String,
    pub followers: u64,
    pub following: u64,
    pub tweets: u64,
    pub likes: u64,
    pub verified: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendItem {
    pub rank: u32,
    pub topic: String,
    pub tweets: String,
    pub category: String,
}
