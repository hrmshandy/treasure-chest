use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NxmUrl {
    pub game: String,
    pub mod_id: u32,
    pub file_id: u32,
    pub key: String,
    pub expires: Option<u64>,
    pub user_id: Option<u32>,
}

#[derive(Debug)]
pub enum NxmError {
    InvalidScheme,
    InvalidFormat,
    UnsupportedGame(String),
    MissingKey,
    InvalidModId,
    InvalidFileId,
    Expired,
    ParseError(String),
}

impl std::fmt::Display for NxmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NxmError::InvalidScheme => write!(f, "Invalid URL scheme (expected nxm://)"),
            NxmError::InvalidFormat => write!(f, "Invalid NXM URL format"),
            NxmError::UnsupportedGame(game) => write!(f, "Game not supported: {}", game),
            NxmError::MissingKey => write!(f, "Missing authentication key"),
            NxmError::InvalidModId => write!(f, "Invalid mod ID format"),
            NxmError::InvalidFileId => write!(f, "Invalid file ID format"),
            NxmError::Expired => write!(f, "Download link has expired. Please download again from Nexus Mods."),
            NxmError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for NxmError {}

impl NxmUrl {
    /// Parse an NXM URL
    /// Format: nxm://stardewvalley/mods/{mod_id}/files/{file_id}?key={key}&expires={timestamp}&user_id={id}
    pub fn parse(url_str: &str) -> Result<Self, NxmError> {
        // Parse URL
        let url = Url::parse(url_str).map_err(|e| NxmError::ParseError(e.to_string()))?;

        // Validate scheme
        if url.scheme() != "nxm" {
            return Err(NxmError::InvalidScheme);
        }

        // Extract game domain
        let game = url
            .host_str()
            .ok_or(NxmError::InvalidFormat)?
            .to_string();

        // Validate game is Stardew Valley
        if game != "stardewvalley" {
            return Err(NxmError::UnsupportedGame(game));
        }

        // Parse path: /mods/{mod_id}/files/{file_id}
        let path_segments: Vec<&str> = url.path().split('/').filter(|s| !s.is_empty()).collect();

        if path_segments.len() != 4
            || path_segments[0] != "mods"
            || path_segments[2] != "files"
        {
            return Err(NxmError::InvalidFormat);
        }

        // Parse mod_id
        let mod_id = path_segments[1]
            .parse::<u32>()
            .map_err(|_| NxmError::InvalidModId)?;

        // Parse file_id
        let file_id = path_segments[3]
            .parse::<u32>()
            .map_err(|_| NxmError::InvalidFileId)?;

        // Parse query parameters
        let mut key: Option<String> = None;
        let mut expires: Option<u64> = None;
        let mut user_id: Option<u32> = None;

        for (param_name, param_value) in url.query_pairs() {
            match param_name.as_ref() {
                "key" => key = Some(param_value.to_string()),
                "expires" => {
                    expires = param_value.parse::<u64>().ok();
                }
                "user_id" => {
                    user_id = param_value.parse::<u32>().ok();
                }
                _ => {} // Ignore unknown parameters
            }
        }

        // Validate key is present
        let key = key.ok_or(NxmError::MissingKey)?;

        if key.is_empty() {
            return Err(NxmError::MissingKey);
        }

        Ok(NxmUrl {
            game,
            mod_id,
            file_id,
            key,
            expires,
            user_id,
        })
    }

    /// Check if the URL has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            expires < now
        } else {
            false // No expiration = never expires
        }
    }

    /// Validate the URL (check expiration and other constraints)
    pub fn validate(&self) -> Result<(), NxmError> {
        if self.is_expired() {
            return Err(NxmError::Expired);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_url_with_all_params() {
        let url = "nxm://stardewvalley/mods/2400/files/9567?key=abc123&expires=1735344000&user_id=12345";
        let nxm = NxmUrl::parse(url).unwrap();

        assert_eq!(nxm.game, "stardewvalley");
        assert_eq!(nxm.mod_id, 2400);
        assert_eq!(nxm.file_id, 9567);
        assert_eq!(nxm.key, "abc123");
        assert_eq!(nxm.expires, Some(1735344000));
        assert_eq!(nxm.user_id, Some(12345));
    }

    #[test]
    fn test_parse_valid_url_without_expiration() {
        let url = "nxm://stardewvalley/mods/2400/files/9567?key=abc123";
        let nxm = NxmUrl::parse(url).unwrap();

        assert_eq!(nxm.expires, None);
        assert!(!nxm.is_expired());
    }

    #[test]
    fn test_parse_rejects_wrong_game() {
        let url = "nxm://skyrim/mods/1234/files/5678?key=test";
        let result = NxmUrl::parse(url);

        assert!(result.is_err());
        match result {
            Err(NxmError::UnsupportedGame(game)) => assert_eq!(game, "skyrim"),
            _ => panic!("Expected UnsupportedGame error"),
        }
    }

    #[test]
    fn test_parse_rejects_missing_key() {
        let url = "nxm://stardewvalley/mods/2400/files/9567";
        let result = NxmUrl::parse(url);

        assert!(result.is_err());
        assert!(matches!(result, Err(NxmError::MissingKey)));
    }

    #[test]
    fn test_parse_rejects_invalid_mod_id() {
        let url = "nxm://stardewvalley/mods/abc/files/9567?key=test";
        let result = NxmUrl::parse(url);

        assert!(result.is_err());
        assert!(matches!(result, Err(NxmError::InvalidModId)));
    }

    #[test]
    fn test_parse_rejects_invalid_file_id() {
        let url = "nxm://stardewvalley/mods/2400/files/xyz?key=test";
        let result = NxmUrl::parse(url);

        assert!(result.is_err());
        assert!(matches!(result, Err(NxmError::InvalidFileId)));
    }

    #[test]
    fn test_expiration_validation() {
        // Create URL that expires in year 2000 (already passed)
        let url = "nxm://stardewvalley/mods/2400/files/9567?key=test&expires=946684800";
        let nxm = NxmUrl::parse(url).unwrap();

        assert!(nxm.is_expired());
        assert!(nxm.validate().is_err());
    }
}
