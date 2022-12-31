pub const CONFIG_FILE_NAME: &str = "spotifyConfig.json";
pub const CACHED_CONFIG_FILE_NAME: &str = "cachedSpotifyConfig.json";

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct SpotifyAutomationModuleConfiguration {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub struct SpotifyAutomationModuleCachedConfiguration {
    pub refresh_token: Option<String>,
}
