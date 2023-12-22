use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Group {
    pub name: String,
    pub dependencies: Vec<String>,
}
