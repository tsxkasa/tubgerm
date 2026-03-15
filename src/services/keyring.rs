use keyring::{Entry, Error};

#[derive(Default, Debug)]
pub struct KeyringService {
    entry: Option<Entry>,
}

impl KeyringService {
    pub fn new(service: &str, uname: &str) -> Result<Self, Error> {
        let service = format!("{}:{}", service, uname);
        let entry = Entry::new(service.as_str(), uname)?;
        Ok(Self { entry: Some(entry) })
    }

    pub fn get_password(&self) -> Result<Option<String>, Error> {
        let entry = match self.entry.as_ref() {
            Some(e) => e,
            None => return Ok(None),
        };

        match entry.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(Error::NoEntry) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set_password(&self, pw: &str) -> Result<(), Error> {
        let entry = match self.entry.as_ref() {
            Some(e) => e,
            None => return Err(Error::NoEntry),
        };

        entry.set_password(pw)
    }

    pub fn entry(&self) -> Result<&Entry, Error> {
        self.entry.as_ref().ok_or(Error::NoEntry)
    }
}
