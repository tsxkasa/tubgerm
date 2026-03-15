use keyring::{Entry, Error};

#[derive(Debug)]
pub struct KeyringService {
    entry: Entry,
}

impl KeyringService {
    pub fn new(service: &str, uname: &str) -> Result<Self, Error> {
        let service = format!("{}:{}", service, uname);
        let entry = Entry::new(service.as_str(), uname)?;
        Ok(Self { entry })
    }

    pub fn get_password(&self) -> Result<Option<String>, Error> {
        match self.entry.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(Error::NoEntry) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set_password(&self, pw: &str) -> Result<(), Error> {
        self.entry.set_password(pw)
    }

    pub fn entry(&self) -> &Entry {
        &self.entry
    }
}
