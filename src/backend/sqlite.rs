//! The sqlite backend just stores all data (including BLOBs) in a single
//! sqlite3 file. SQLite is great at storing lots of small blobs this way,
//! but may perform poorly for lots of large files.
//! 
//! Mostly, this makes data management trivial since it's all in one file.
//! But if performance is an issue we can implement a different backend.

use crate::backend::{self, Backend, Signature, Hash};

use failure::{Error, bail};
use rusqlite::{params, OptionalExtension};

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
            conn: rusqlite::Connection::open(&self.file_path)?
        };
        Ok(Box::new(conn))
    }
}

pub(crate) struct Connection
{
    conn: rusqlite::Connection,
}

impl Connection
{
    fn setup_new(&self) -> Result<(), Error>
    {
        self.run("CREATE TABLE version (version INTEGER)")?;
        self.run("INSERT INTO version VALUES(1)")?;

        self.run("
            CREATE TABLE signature(
                -- A signature is the core data structure of FeoBlog.
                -- It's a cryptographic signature of CBOR metadata about
                -- a paritcular 'post'. Together, it proves that a user
                -- made a post.
                user_id BLOB -- ed25519 public signing key
                , signature BLOB -- detached signature of the data
                , unix_utc INTEGER -- UTC timestamp signed integer of when this was created
                , received_utc INTEGER -- When this signature was received (vs. created)
                , cbor_metadata BLOB -- metadata about this content
            )
        ")?;
        self.run("
            CREATE UNIQUE INDEX signature_primary_idx
            ON signature(user_id, signature)
        ")?;
        self.run("
            CREATE INDEX signature_user_chrono_idx
            ON signature(user_id, unix_utc)
        ")?;
        self.run("
            CREATE INDEX signature_user_chrono_received_idx
            ON signature(user_id, received_utc)
        ")?;
        self.run("
            CREATE INDEX signature_unix_utc_idx
            ON signature(unix_utc)
        ")?;
        self.run("
            CREATE INDEX signature_received_utc_idx
            ON signature(received_utc)
        ")?;

        self.run("
            CREATE TABLE blob(
                -- A content-addressable store for many kinds of data.
                hash BLOB PRIMARY KEY, -- multihash of the data.
                data BLOB
            )
        ")?; 


        // TODO: Cache tables for quick lookups of things.

        Ok(())
    }

    fn run(&self, sql: &str) -> Result<(), rusqlite::Error>
    {
        self.conn.execute(sql, params![])?;
        Ok(())
    }

    fn get_version(&self) -> Result<Option<u32>, Error>
    {
        let table_count: u32  = self.conn.prepare(
            "SELECT count()
            FROM sqlite_master
            WHERE type = 'table'
            AND name = 'version'
            "
        )?.query_row(
            params![],
            |row|  Ok(row.get(0)?)
        )?;

        if table_count == 0 {
            return Ok(None);
        }
        if table_count > 1 {
            bail!("Found {} versions in the version table.", table_count);
        }

        let  version = self.conn.prepare(
            "SELECT MAX(version) from version"
        )?.query_row(
            params![],
            |row| Ok(row.get(0)?)
        )?;

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
        if version == CURRENT_VERSION {
            return Ok(());
        }
        if version > CURRENT_VERSION {
            bail!(
                "DB version ({}) newer than current version ({})",
                version,
                CURRENT_VERSION
            );
        }

        // TODO:
        bail!("DB version {} is unknown. Migration not implemented.", version);
    }

    // Reads an entire blob into memory. TODO: Make a streaming version.
    fn get_blob(&self, hash: &backend::Hash) -> Result<Option<Vec<u8>>, Error>
    {
        let mut stmt = self.conn.prepare("
            SELECT data FROM blob WHERE hash = ?
        ")?;

        let blob = stmt.query_row(
            params![hash.as_bytes()],
            |row| Ok(row.get(0)?)
        ).optional()?;

        Ok(blob)
    }

    fn get_signature(&self, _key: &[u8]) -> Result<Option<Signature>, Error>
    {
        Ok(None)
    }

    // Make a streaming version.
    fn save_blob(&self, data: &[u8]) -> Result<backend::Hash, Error>
    {
        let hash = backend::Hash::calculate(data);
        let mut stmt = self.conn.prepare("
            INSERT OR IGNORE INTO blob(hash, data)
            VALUES(?, ?)
        ")?;

        stmt.insert(
            params![hash.as_bytes(), data]
        )?;

        Ok(hash)
    }

    fn get_hashes(&self) -> Result<Vec<backend::Hash>, Error>
    {
        let mut stmt = self.conn.prepare("
            SELECT hash
            FROM blob
            ORDER BY hash
            LIMIT 10000
        ")?;

        let hash_iter = stmt.query_map(
            params![],
            |row| Ok(
                backend::Hash{multihash: row.get(0)?}
            )
        )?;

        let hashes: Result<Vec<backend::Hash>, rusqlite::Error> = hash_iter.collect();
        Ok(hashes?)
    }

}