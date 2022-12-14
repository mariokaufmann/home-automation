#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SetStatePayload {
    pub on: bool,
}
