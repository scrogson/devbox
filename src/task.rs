use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Task {
    pub name: String,
    pub description: String,
    pub exec: Vec<String>,
}
