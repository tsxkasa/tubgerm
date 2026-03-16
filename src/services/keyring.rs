use color_eyre::eyre::Result;
use keyring::Entry;

#[derive(Default, Debug)]
pub struct KeyringService {
    entry: Option<Entry>,
}

impl KeyringService {
    pub fn new(service: &str, uname: &str) -> Result<Self> {
        let service = format!("{}:{}", service, uname);
        let entry = Entry::new(service.as_str(), uname)?;
        Ok(Self { entry: Some(entry) })
    }

    pub fn get_password(&self) -> Result<Option<String>> {
        let entry = match self.entry.as_ref() {
            Some(e) => e,
            None => return Ok(None),
        };

        match entry.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_password(&self, pw: &str) -> Result<()> {
        let entry = match self.entry.as_ref() {
            Some(e) => e,
            None => return Err(keyring::Error::NoEntry.into()),
        };

        entry.set_password(pw).map_err(Into::into)
    }

    pub fn entry(&self) -> Result<&Entry> {
        self.entry.as_ref().ok_or(keyring::Error::NoEntry.into())
    }
}
