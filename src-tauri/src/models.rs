use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModManifest {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Author", default = "default_author")]
    pub author: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "UniqueID")]
    pub unique_id: String,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Dependencies")]
    pub dependencies: Option<Vec<ModDependency>>,
    #[serde(rename = "ContentPackFor")]
    pub content_pack_for: Option<ContentPackInfo>,
}

fn default_author() -> String {
    "Unknown".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentPackInfo {
    #[serde(rename = "UniqueID")]
    pub unique_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModDependency {
    #[serde(rename = "UniqueID")]
    pub unique_id: String,
    #[serde(rename = "IsRequired")]
    pub is_required: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub unique_id: String,
    pub description: Option<String>,
    pub path: String,
    pub is_enabled: bool,
}
