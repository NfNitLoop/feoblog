use crate::backend::{self, Backend, Signature, Hash};

use failure::{Error, bail};

#[derive(Clone)]
pub(crate) struct Factory
{
    file_path: String
}

impl Factory {
    pub fn new(file_path: String) -> Self
    {
        Factory{file_path}
    }
}

impl backend::Factory for Factory
{
    fn open(&self) -> Result<Box<dyn backend::Backend>, Error>
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
        self.run("CREATE TABLE version (version INTEGER)")?;
        self.run("INSERT INTO version VALUES(1)")?;

        self.run("
            CREATE TABLE signatures(
                user_id BLOB -- ed25519 public signing key
                , metadata_hash BLOB -- multihash reference to the data. 
                , signature BLOB -- signature on the metadata_hash.
            )
        ")?;
        self.run("CREATE UNIQUE INDEX signatures_signature_idx ON signatures(signature)")?;
        self.run("CREATE INDEX signatures_user_idx ON signatures(user_id)")?;
        self.run("CREATE INDEX signatures_hash_idx ON signatures(metadata_hash)")?;

        // content-addressable store
        self.run("
            CREATE TABLE blobs(
                hash BLOB PRIMARY KEY, -- multihash of the data.
                data BLOB
            )
        ")?; 

        // TODO: Cache tables.

        Ok(())
    }

    fn run(&self, sql: &str) -> Result<(), sqlite::Error>
    {
        self.conn.execute(sql)
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

impl backend::Backend for Connection
{
    fn setup(&self) -> Result<(), Error>
    {
        let version = match self.get_version()? {
            None => {
                return self.setup_new();
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

    fn get_blob(&self, hash: &backend::Hash) -> Result<Option<Vec<u8>>, Error>
    {
        let mut stmt = self.conn.prepare("
            SELECT data FROM blobs WHERE hash = ?
        ")?;
        stmt.bind(1, hash.as_bytes())?;
        if let sqlite::State::Row = stmt.next()? {
            return Ok(Some(stmt.read(0)?));
        }
        Ok(None)
    }

    fn get_signature(&self, key: &[u8]) -> Result<Option<Signature>, Error>
    {
        Ok(None)
    }

    fn save_blob(&self, data: &[u8]) -> Result<backend::Hash, Error>
    {
        let hash = backend::Hash::calculate(data);
        let mut stmt = self.conn.prepare("
            INSERT OR IGNORE INTO blobs(hash, data)
            VALUES(?, ?)
        ")?;
        stmt.bind(1, hash.as_bytes())?;
        stmt.bind(2, data)?;
        let result = stmt.next()?;
        if result != sqlite::State::Done {
            bail!("Unexpected state: {:?}", result);
        }
        Ok(hash)
    }

    fn get_hashes(&self) -> Result<Vec<backend::Hash>, Error>
    {
        let mut stmt = self.conn.prepare("
            SELECT hash
            FROM blobs
            ORDER BY hash
            LIMIT 10000
        ")?;
        let mut hashes = Vec::new();
        while sqlite::State::Row == stmt.next()? {
            hashes.push(
                backend::Hash{multihash: stmt.read(0)?}
            );
        }
        Ok(hashes)
    }

}