//! The sqlite backend just stores all data (including BLOBs) in a single
//! sqlite3 file. SQLite is great at storing lots of small blobs this way,
//! but may perform poorly for lots of large files.
//! 
//! Mostly, this makes data management trivial since it's all in one file.
//! But if performance is an issue we can implement a different backend.

use crate::backend::{self, UserID, ItemRow, Timestamp, ServerUser};

use failure::{Error, bail};
use rusqlite::{params, OptionalExtension, Row};

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
        self.run("
            CREATE TABLE version (
                -- The current version of the database schema.
                version INTEGER
            )
        ")?;
        self.run("INSERT INTO version VALUES(2)")?;

        self.run("
            CREATE TABLE item(
                -- An Item is the core data structure of FeoBlog.
                -- It is a BLOB of protobuf v3 bytes defining an item in a
                -- user's collection of items
                bytes BLOB

                -- An item must be accompanied by a nacl public key (user_id)
                -- and (detached) signature so that its authenticity can be
                -- verified.
                , user_id BLOB
                , signature BLOB

                -- A copy of the signed timestamp from within `bytes`
                -- this allows for sorting queries by timestamp.
                , unix_utc_ms INTEGER

                -- The date this item was received by this server. May differ
                -- from above.
                , received_utc_ms INTEGER
            )
        ")?;
        self.run("
            CREATE UNIQUE INDEX item_primary_idx
            ON item(user_id, signature)
        ")?;
        self.run("
            CREATE INDEX item_user_chrono_idx
            ON item(user_id, unix_utc_ms)
        ")?;
        self.run("
            CREATE INDEX item_user_chrono_received_idx
            ON item(user_id, received_utc_ms)
        ")?;
        self.run("
            CREATE INDEX item_unix_utc_idx
            ON item(unix_utc_ms)
        ")?;
        self.run("
            CREATE INDEX item_received_utc_idx
            ON item(received_utc_ms)
        ")?;

        self.run("
            CREATE TABLE server_user(
                -- These users have been granted direct access to the server.
                
                user_id BLOB

                -- Information about this user.
                -- Not displayed on the web UI, just here to let the server
                -- admin leave a human-readable note about who this user is.
                , notes TEXT

                -- bool 0/1 -- should this user's posts appear on the home page
                -- of this server?
                , on_homepage INTEGER

                -- How many bytes will the server cache for this user?
                -- 0 = unlimited.
                , max_bytes INTEGER 
            )
        ")?;

        self.run("
            CREATE UNIQUE INDEX server_user_primary_idx
            ON server_user(user_id)
        ")?;

        self.run("
            CREATE INDEX server_user_homepage_idx
            ON server_user(on_homepage, user_id)
        ")?;


        // TODO: a "profile" table which tracks users' latest profile.
        // TODO: a "follow" table which shows which users are followed by
        // server_users

        // self.run("
        //     CREATE TABLE blob(
        //         -- A content-addressable store for many kinds of data.
        //         hash BLOB PRIMARY KEY, -- multihash of the data.
        //         data BLOB
        //     )
        // ")?; 


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

        // TODO: Oops, meant to count rows here, not tables. 
        // if table_count > 1 {
        //     bail!("Found {} versions in the version table.", table_count);
        // }

        let  version = self.conn.prepare(
            "SELECT MAX(version) from version"
        )?.query_row(
            params![],
            |row| Ok(row.get(0)?)
        )?;

        Ok(version)
    }
}

const CURRENT_VERSION: u32 = 2;

impl backend::Backend for Connection
{

    fn setup(&self) -> Result<(), Error>
    {
        let version = match self.get_version()? {
            None => {
                // TODO: This shouldn't be automatic, should force user to
                // explicitly create a new data store.
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

    // fn get_hashes(&self) -> Result<Vec<backend::Hash>, Error>
    // {
    //     let mut stmt = self.conn.prepare("
    //         SELECT hash
    //         FROM blob
    //         ORDER BY hash
    //         LIMIT 10000
    //     ")?;

    //     let hash_iter = stmt.query_map(
    //         params![],
    //         |row| Ok(
    //             backend::Hash{multihash: row.get(0)?}
    //         )
    //     )?;

    //     let hashes: Result<Vec<backend::Hash>, rusqlite::Error> = hash_iter.collect();
    //     Ok(hashes?)
    // }

    fn save_user_item(&self, _:ItemRow)
    -> Result<(), Error>
    {
        todo!() 
    }

    fn homepage_items(&self, _:Timestamp)
    -> Result<Vec<backend::ItemRow>, Error>
    {
        todo!() 
    }

    fn user_items(&self, _: &UserID, _:Timestamp)
    -> Result<Vec<ItemRow>, Error>
    {
        todo!() 
    }

    fn server_user(&self, user: &UserID)
    -> Result<Option<backend::ServerUser>, Error> 
    { 
        let mut stmt = self.conn.prepare("
            SELECT notes, on_homepage
            FROM server_user
            WHERE user_id = ?
        ")?;

        let to_server_user = |row: &Row<'_>| {
            let on_homepage: isize = row.get(1)?;
             Ok(
                 ServerUser {
                    user: (*user).clone(),
                    notes: row.get(0)?,
                    on_homepage: on_homepage != 0,
                }
            )
        };

        let item = stmt.query_row(
            params![user.bytes()],
            to_server_user,
        ).optional()?;

        Ok(item)

    }
}