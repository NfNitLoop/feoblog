use crate::backend as traits;

use failure::{Error, bail};

#[derive(Clone)]
pub(crate) struct Factory
{
    file_path: String
}

impl Factory {
    pub fn new(file_path: String) -> Self
    {
        Factory{file_path: file_path}
    }
}

impl traits::Factory for Factory
{
    fn open(&self) -> Result<Box<dyn traits::Backend>, Error>
    {
        let conn = Connection{
            conn: sqlite::open(&self.file_path)?
        };
        Ok(Box::new(conn))
    }
}

pub(crate) struct Connection
{
    conn: sqlite::Connection,
}

impl Connection
{
    fn setup_new(&self) -> Result<(), Error>
    {
        self.conn.execute("CREATE TABLE version (version INTEGER)")?;
        self.conn.execute("INSERT INTO version VALUES(1)")?;
        self.conn.execute(
            "CREATE TABLE blobs (hash BLOB, value BLOB)"
        )?;
        self.conn.execute("CREATE UNIQUE INDEX blobs_hash_idx ON blobs (hash)")?;

        Ok(())
    }

    fn get_version(&self) -> Result<Option<u32>, Error>
    {
        let mut cursor = self.conn.prepare(
            "SELECT count()
            FROM sqlite_master
            WHERE type = 'table'
            AND name = 'version'
            "
        )?.cursor();
        let row = match cursor.next()?
        {
            Some(row) => row,
            None => bail!("No rows back from DB?")
        };

        let table_count = row[0].as_integer();
        match table_count {
            None => bail!("Error counting version table."),
            Some(0) => { return Ok(None); } // No version yet.
            Some(count) if count > 1 => {
                bail!("Found {} version tables!?", count);
            }
            _ => {} // OK
        }

        let mut cursor = self.conn.prepare(
            "SELECT MAX(version) from version"
        )?.cursor();

        let row = cursor.next()?;
        let row = match row {
            None => bail!("No version in version table."),
            Some(row) => row
        };

        let version = row[0].as_integer().map(|x| x as u32);
        Ok(version)
    }
}

const CURRENT_VERSION: u32 = 1;

impl traits::Backend for Connection
{
    fn setup(&self) -> Result<(), Error>
    {
        let version = match self.get_version()? {
            None => {
                self.setup_new();
                return Ok(());
            },
            Some(version) => version
        };
        if (version == CURRENT_VERSION) {
            return Ok(());
        }
        if (version > CURRENT_VERSION) {
            bail!(
                "DB version ({}) newer than current version ({})",
                version,
                CURRENT_VERSION
            );
        }

        // TODO:
        bail!("DB version {} is unknown. Migration not implemented.", version);
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error>
    {
        let mut stmt = self.conn.prepare("
            SELECT value FROM blobs WHERE hash = ?
        ")?;
        stmt.bind(1, key);
        if let sqlite::State::Row = stmt.next()? {
            return Ok(Some(stmt.read(0)?));
        }
        Ok(None)
    }
}