#[derive(Debug)]
pub struct Report {
    pub address: String,
    pub balance: String,
    pub description: String,
    pub potential_threat: Vec<String>,
}
