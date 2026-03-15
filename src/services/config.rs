use crate::core::user_data::UserData;

#[derive(Debug)]
pub struct ConfigService {
    user_data: UserData,
}

impl ConfigService {
    pub fn new(url: &str, uname: &str) -> Self {
        Self {
            user_data: UserData::new(url.to_string(), uname.to_string()),
        }
    }
}
