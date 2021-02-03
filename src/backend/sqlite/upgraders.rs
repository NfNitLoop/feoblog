//! Types that know how to upgrade the SQLite database.

use failure::{Error, bail};
use protobuf::Message;
use rusqlite::params;

use crate::{backend::{ItemRow, RowCallback, Signature, UserID}, protos::Item};

use super::{CURRENT_VERSION, Connection, ReplyRow, save_reply_rows};

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


/// New `reply` table to track comments.
/// (and eventually also "reply posts" and other things that may be able to reply)
struct From3To4;
impl Upgrader for From3To4 {
    fn from_version(&self) -> u32 { 3 }
    fn to_version(&self) -> u32 { 4 }

    fn upgrade(&self, conn: &Connection) -> Result<(), Error> {
        conn.run("
            CREATE TABLE reply (
                -- Tracks Items that 'reply to' other items.

                from_user_id BLOB,
                from_signature BLOB,

                to_user_id BLOB,
                to_signature BLOB
            )
        ")?;

        conn.run("
            CREATE INDEX reply_to_idx
            ON reply(to_user_id, to_signature)
        ")?;

        // TODO: Newer rusqlite supports u64 & usize:
        let item_count: u32 = conn.conn.query_row(
            "SELECT COUNT(*) FROM item",
            params![],
            |row| Ok(row.get(0)?)
        )?;

        if item_count > 1000 {
            println!("Scanning {} items for replies. This may take a some time.", item_count);
        }

        let mut pager = ItemPager::new();
        
        // If SQLite hasn't enabled WAL (not the default, & not supported
        // everywhere) then we can't read & write from the DB at the same time.
        // Batch up rows to write here, and periodically write them:
        let mut reply_tos = Vec::<ReplyRow>::new();
        let max_replies = 1000;
        
        while !pager.done {
            pager.iterate(conn, &mut |row| {
                let mut item = Item::new();
                item.merge_from_bytes(row.item_bytes.as_slice())?;

                if !item.has_comment() {
                    // Currently, only comments have replies.
                    // If/when we add new reply types in the future, we'll need
                    // to run a similar migration for those in a separate
                    // migration.  In the meantime:
                    return Ok(true) // continue
                }

                let comment = item.get_comment();
                
                let reply = ReplyRow {
                    from_user_id: row.user.clone(),
                    from_signature: row.signature.clone(),
                    to_user_id: UserID::from_vec(comment.get_reply_to().get_user_id().get_bytes().into())?,
                    to_signature: Signature::from_vec(comment.get_reply_to().get_signature().get_bytes().into())?,
                };
                reply_tos.push(reply);

                Ok(reply_tos.len() < max_replies)
            })?;

            // Write cached reply_tos to the database:
            save_reply_rows(&*conn.conn, reply_tos.as_slice())?;
            reply_tos.clear();
        }

        conn.set_version(self.to_version())?;
        Ok(())
    }
}

struct ItemPager {
    after_uid: Option<UserID>,
    after_sig: Option<Signature>,
    done: bool,
}


impl ItemPager {
    fn new() -> Self {
        Self {
            after_uid: None,
            after_sig: None,
            done: false,
        }
    }

    /// Iterate through the Connection.all_items() results, and store the iteration point to resume.
    fn iterate<'a>(&mut self, conn: &Connection, cb: RowCallback<'a, ItemRow>) -> Result<(), Error> {
        let mut new_uid = None;
        let mut new_sig = None;

        // If we get zero results, default to "done":
        let mut done = true;

        let mut myCb = |row: ItemRow| {
            new_uid = Some(row.user.clone());
            new_sig = Some(row.signature.clone());

            let result = cb(row);
            if let Ok(should_continue) = result {
                // If we tell the function we want more, but it never calls it again, then we're done iterating:
                done = should_continue;
            }
            return result;
        };


        conn.all_items(&self.after_uid, &self.after_sig, &mut myCb)?;

        self.after_uid = new_uid;
        self.after_sig = new_sig;
        self.done = done;

        Ok(())
    }
}

