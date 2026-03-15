use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Config {
    pub credentials: Credentials,
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Credentials {
    pub server: String,
    pub username: String,
}
