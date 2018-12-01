#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebToken {
    jwt: String,
}

impl WebToken {
    pub fn issue() -> Self {
        WebToken {
            jwt: "testtoken".to_owned(),
        }
    }
}
