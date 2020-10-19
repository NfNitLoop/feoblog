//! The sqlite backend just stores all data (including BLOBs) in a single
//! sqlite3 file. SQLite is great at storing lots of small blobs this way,
//! but may perform poorly for lots of large files.
//! 
//! Mostly, this makes data management trivial since it's all in one file.
//! But if performance is an issue we can implement a different backend.

use crate::protos::Item;
use rusqlite::NO_PARAMS;
use crate::backend::FnIter;
use crate::backend::{self, UserID, Signature, ItemRow, Profile, ItemProfileRow, Timestamp, ServerUser, Backend};

use failure::{Error, bail, ResultExt};
use rusqlite::{params, OptionalExtension, Row};

const CURRENT_VERSION: u32 = 3;


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
        self.run("INSERT INTO version VALUES(3)")?;

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


        self.run("
            CREATE TABLE follow(
                -- Lists which users follow which other users.
                -- Always represents the latest Profile saved by a user.
                source_user_id BLOB,
                followed_user_id BLOB,
                display_name TEXT
            )
        ")?;

        self.run("
            CREATE UNIQUE INDEX follow_primary_idx
            ON follow(source_user_id, followed_user_id)
        ")?;

        self.run("
            CREATE TABLE profile(
                -- Always contains a reference to the latest profile uploaded by a user
                user_id BLOB,
                signature BLOB,
                display_name TEXT
            )
        ")?;

        self.run("
            CREATE UNIQUE INDEX profile_primary_idx
            ON profile(user_id)
        ")?;


        // TODO: Store file attachments, etc:
        // self.run("
        //     CREATE TABLE blob(
        //         -- A content-addressable store for many kinds of data.
        //         hash BLOB PRIMARY KEY, -- multihash of the data.
        //         data BLOB
        //     )
        // ")?; 


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

        let  version = self.conn.prepare(
            "SELECT MAX(version) from version"
        )?.query_row(
            params![],
            |row| Ok(row.get(0)?)
        )?;

        Ok(version)
    }

    /// We're saving a profile. If it's new, update the profile and follow tables.
    /// Expects to be run from within a transaction.
    fn update_profile(&self, item_row: &ItemRow, item: &Item) -> Result<(), Error> {

        // TODO: Really could just get the profile timestamp from the DB w/o deserializing.
        let profile = self.get_profile(&item_row.user)?;

        // Never replace a newer profile's metadata:
        if let Some(profile) = profile {
            if profile.timestamp.unix_utc_ms >= item.timestamp_ms_utc {
                return Ok(())
            }
        }

        // Replace all follows with new ones listed in the profile:
        self.conn.execute("DELETE FROM follow WHERE source_user_id = ?", params![item_row.user.bytes()])?;

        // Behavior is undefined if duplicate follows exist in a Profile.
        let mut add_follow = self.conn.prepare("
            INSERT OR REPLACE INTO follow (source_user_id, followed_user_id, display_name)
            VALUES (?, ?, ?)
        ")?;

        for follow in item.get_profile().get_follows() {
            add_follow.execute(params![
                item_row.user.bytes(),
                follow.get_user().get_bytes(),
                follow.get_display_name(),
            ])?;
        }

        let mut add_profile = self.conn.prepare("
            INSERT OR REPLACE INTO profile(user_id, signature, display_name)
            VALUES (?,?,?)
        ")?;
        add_profile.execute(params![
            item_row.user.bytes(),
            item_row.signature.bytes(),
            item.get_profile().get_display_name()
        ])?;

        Ok(())
    }

    // TODO: Should move to the Backend trait:
    fn get_profile(&self, user: &UserID) -> Result<Option<ItemRow>, Error> {

        // TODO: I'm not crazy about making 2 queries here instead of a join, but it lets me
        // re-use the user_item() loading logic.
        let mut find_profile = self.conn.prepare("
            SELECT user_id, signature
            FROM profile
            WHERE user_id = ?
        ")?;

        let mut rows = find_profile.query(params![user.bytes()])?;
        let row = match rows.next()? {
            None => return Ok(None),
            Some(row) => row,
        };

        let user_id: Vec<u8> = row.get(0)?;
        let signature: Vec<u8> = row.get(1)?;

        let user_id = UserID::from_vec(user_id)?;
        let signature = Signature::from_vec(signature)?;

        self.user_item(&user_id, &signature)
    }
}


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

    fn homepage_items<'a>(
        &self,
        before: Timestamp,
        callback: &'a mut dyn FnMut(ItemProfileRow) -> Result<bool,Error>
    ) -> Result<(), Error> {
        let mut stmt = self.conn.prepare("
            SELECT
                user_id
                , i.signature
                , unix_utc_ms
                , received_utc_ms
                , bytes
                , p.signature
                , p.display_name
            FROM item AS i
            LEFT OUTER JOIN profile AS p USING (user_id)
            WHERE unix_utc_ms < ?
            AND user_id IN (
                SELECT user_id
                FROM server_user
                WHERE on_homepage = 1
            )
            ORDER BY unix_utc_ms DESC
        ")?;

        let mut rows = stmt.query(params![
            before.unix_utc_ms,
        ])?;

        let to_item_profile_row = |row: &Row<'_>| -> Result<ItemProfileRow, Error> {
            let profile = match (row.get(5)?, row.get(6)?) {
                (Some(signature), Some(display_name)) => Some(
                        Profile{
                        signature: Signature::from_vec(signature)?,
                        display_name,                    
                    }
                ),
                _ => None
            };

            let item = ItemRow{
                user: UserID::from_vec(row.get(0)?)?,
                signature: Signature::from_vec(row.get(1)?)?,
                timestamp: Timestamp{ unix_utc_ms: row.get(2)? },
                received: Timestamp{ unix_utc_ms: row.get(3)? },
                item_bytes: row.get(4)?,
            };

            Ok(ItemProfileRow{item, profile})
        };

        while let Some(row) = rows.next()? {
            let item = to_item_profile_row(row)?;
            let result = callback(item)?;
            if !result { break; }
        }

        Ok( () )
    }

    fn user_items<'a>(
        &self,
        user: &UserID,
        before: Timestamp,
        callback: &'a mut dyn FnMut(ItemRow) -> Result<bool,Error>
    ) -> Result<(), Error> {
        let mut stmt = self.conn.prepare("
            SELECT
                user_id
                , i.signature
                , unix_utc_ms
                , received_utc_ms
                , bytes
            FROM item AS i
            WHERE
                unix_utc_ms < ?
                AND user_id = ?
            ORDER BY unix_utc_ms DESC
        ")?;

        let mut rows = stmt.query(params![
            before.unix_utc_ms,
            user.bytes(),
        ])?;

        let convert = |row: &Row<'_>| -> Result<ItemRow, Error> {
            let item = ItemRow{
                user: UserID::from_vec(row.get(0)?)?,
                signature: Signature::from_vec(row.get(1)?)?,
                timestamp: Timestamp{ unix_utc_ms: row.get(2)? },
                received: Timestamp{ unix_utc_ms: row.get(3)? },
                item_bytes: row.get(4)?,
            };

            Ok(item)
        };

        while let Some(row) = rows.next()? {
            let item = convert(row)?;
            let result = callback(item)?;
            if !result { break; }
        }

        Ok( () )
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
                    user: user.clone(),
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

    fn server_users<'a>(&self, cb: FnIter<'a, ServerUser>) -> Result<(), Error> {

        let mut stmt = self.conn.prepare("
            SELECT 
                user_id
                , notes
                , on_homepage
            FROM server_user
            ORDER BY on_homepage, user_id
        ")?;

        let mut rows = stmt.query(NO_PARAMS)?;

        while let Some(row) = rows.next()? {
            let on_homepage: isize = row.get(2)?;
            let on_homepage = on_homepage != 0;

            let user = ServerUser {
                user: UserID::from_vec(row.get(0)?).compat()?,
                notes: row.get(1)?,
                on_homepage,
            };
            let more = cb(user)?;
            if !more {break;}
        }

        Ok(())
    }
    
    
    fn user_item_exists(&self, user: &UserID, signature: &Signature) -> Result<bool, Error> { 
        let mut stmt = self.conn.prepare("
            SELECT COUNT(*)
            FROM item
            WHERE user_id = ?
            AND signature = ?
        ")?;

        let count: u32 = stmt.query_row(
            params![
                user.bytes(),
                signature.bytes(),
            ],
            |row| { Ok(row.get(0)?) }
        )?;

        if count > 1 {
            bail!("Found {} matches!? (user_id,signature) should be unique!", count);
        }

        Ok(count > 0)
    }

    fn user_item(&self, user: &UserID, signature: &Signature) -> Result<Option<ItemRow>, Error> { 
        let mut stmt = self.conn.prepare("
            SELECT
                user_id
                , signature
                , unix_utc_ms
                , received_utc_ms
                , bytes
            FROM item
            WHERE user_id = ?
            AND signature = ?
        ")?;

        let mut rows = stmt.query(params![
            user.bytes(),
            signature.bytes(),
        ])?;

        let row = match rows.next()? {
            None => return Ok(None),
            Some(row) => row,
        };

        let item = ItemRow{
            user: UserID::from_vec(row.get(0)?)?,
            signature: Signature::from_vec(row.get(1)?)?,
            timestamp: Timestamp{ unix_utc_ms: row.get(2)? },
            received: Timestamp{ unix_utc_ms: row.get(3)? },
            item_bytes: row.get(4)?,
        };

        if rows.next()?.is_some() {
            bail!("Found multiple matching rows!? (user_id,signature) should be unique!");
        }

        Ok(Some(item))
    }

    fn save_user_item(&self, row: &ItemRow, item: &Item) -> Result<(), Error>
    {
        let tx = self.conn.unchecked_transaction()?;

        let stmt = "
            INSERT INTO item (
                user_id
                , signature
                , unix_utc_ms
                , received_utc_ms
                , bytes
            ) VALUES (?, ?, ?, ?, ?);
       ";

        self.conn.execute(stmt, params![
            row.user.bytes(),
            row.signature.bytes(),
            row.timestamp.unix_utc_ms,
            row.received.unix_utc_ms,
            row.item_bytes.as_slice(),
        ])?;

        if item.has_profile() {
            self.update_profile(row, item)?;
        }

        tx.commit()?;

        Ok(())
    }

    fn add_server_user(&self, server_user: &ServerUser) -> Result<(), Error> {

        let stmt = "
            INSERT INTO server_user(user_id, notes, on_homepage)
            VALUES (?,?,?)
        ";

        let on_homepage = if server_user.on_homepage { 1 } else { 0 };

        self.conn.execute(stmt, params![
            server_user.user.bytes(),
            server_user.notes.as_str(),
            on_homepage
        ])?;

        Ok(())
    }
}