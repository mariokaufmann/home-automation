#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub server_ip: String,
    pub server_port: u32,
    pub device_id: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            server_ip: "127.0.0.1".to_owned(),
            server_port: 80,
            device_id: String::from("default_device_id"),
        }
    }
}
