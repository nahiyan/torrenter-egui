#[derive(Clone)]
pub struct Peer {
    pub region: String,
    pub client: String,
    pub ip_address: String,
    pub progress: f32,
    pub download_rate: i64,
    pub upload_rate: i64,
}
