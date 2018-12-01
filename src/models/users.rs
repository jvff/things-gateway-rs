use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Users {
    users: BTreeMap<usize, String>,
}

impl Users {
    pub fn count(&self) -> usize {
        self.users.len()
    }
}
