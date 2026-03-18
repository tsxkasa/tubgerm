use color_eyre::eyre::Result;
use keyring::Entry;

#[derive(Debug)]
pub struct KeyringService {
    entry: Entry,
}

impl KeyringService {
    pub fn new(service: &str, uname: &str) -> Result<Self> {
        let serv = format!("org.tubgerm.{}@{}", uname, service);
        let entry = Entry::new(serv.as_str(), uname)?;
        Ok(Self { entry })
    }

    pub fn delete_credential(&self) -> Result<()> {
        self.entry.delete_credential()?;
        Ok(())
    }

    pub fn get_password(&self) -> Result<Option<String>> {
        match self.entry.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_password(&self, pw: &str) -> Result<()> {
        self.entry.set_password(pw).map_err(Into::into)
    }

    pub fn entry(&self) -> &Entry {
        &self.entry
    }
}
