//! Types that know how to upgrade the SQLite database.

use failure::{Error, bail};

use super::{CURRENT_VERSION, Connection};

pub(crate) struct Upgraders {
    upgraders: Vec<Box<dyn Upgrader>>
}

impl Upgraders {
    pub fn new() -> Self {
        Self { upgraders: vec![
            Box::new(From3To4)
        ]}
    }

    pub fn upgrade(&self, conn: &Connection) -> Result<(), Error> {
        let mut current_version = conn.get_version()?; 
        while current_version < CURRENT_VERSION {
            let upgrader = self.upgrader_from(current_version)?;
            println!("Upgrading from db version {} to {} ...", current_version, upgrader.to_version());
            upgrader.upgrade(conn)?;
            current_version = conn.get_version()?;
            if current_version != upgrader.to_version() {
                bail!("Upgrader failed to upgrade to advertised version: {}, still {}", upgrader.to_version(), current_version);
            }
        }

        println!("Upgrade complete. Current database version: {}", current_version);

        Ok(())
    }

    fn upgrader_from(&self, version: u32) -> Result<&dyn Upgrader, Error> {
        for upgrader in &self.upgraders {
            if upgrader.from_version() == version {
                return Ok(upgrader.as_ref())
            }
        }
        bail!("Couldn't find a way to upgrade from version {}", version);
    }
}


trait Upgrader {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn upgrade(&self, conn: &Connection) -> Result<(), Error>;
}


struct From3To4;
impl Upgrader for From3To4 {
    fn from_version(&self) -> u32 { 3 }
    fn to_version(&self) -> u32 { 4 }

    fn upgrade(&self, conn: &Connection) -> Result<(), Error> {
        bail!("Not yet implemented")
    }
}