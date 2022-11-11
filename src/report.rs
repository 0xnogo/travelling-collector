use std::fmt::Display;

#[derive(Debug)]
pub struct Report {
    pub address: String,
    pub balance: String,
    pub description: String,
    pub potential_threat: Vec<String>,
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut threats = String::new();

        for i in &self.potential_threat {
            threats.push_str(&format!("====>{i}\n"));
        }

        writeln!(
            f,
            "{} with {}\nRisk: {}\nThreats:\n{}\n\n",
            self.address, self.balance, self.description, threats
        )
    }
}
