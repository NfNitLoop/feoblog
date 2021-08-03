#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables))]
#![deny(unknown_lints)]
#![deny(unused_must_use)]

#[cfg(test)]
mod tests;

use crate::backend::{Factory, PruneOpts, ServerUser, UserID, sqlite};
use anyhow::{Error, bail};
use structopt::StructOpt;

mod backend;
mod markdown;
mod protos;
mod server;


fn main() -> Result<(), Error> {
    let command = Command::from_args();
    use Command::*;

    match command {
        Serve(command) => server::serve(command)?,
        User(command) => command.main()?,
        Db(command) => command.main()?,
    };

    Ok(())
}

#[derive(StructOpt, Debug)]
#[structopt(
    name="feoblog",
    about="A distributed P2P blog system.",
)]
enum Command
{
    #[structopt(name="serve")]
    /// Start a server.
    Serve(ServeCommand),

    /// User administration commands
    User(UserCommand),

    /// Database administration commands
    Db(DbCommand),
}

#[derive(StructOpt, Debug, Clone)]

struct ServeCommand {
    #[structopt(flatten)]
    backend_options: BackendOptions,

    /// Should we open a browser window?
    #[structopt(long)]
    open: bool,

    /// Bind to this local address.
    /// If unspecified, will try to bind to some port on localhost.
    #[structopt(long="bind")]
    binds: Vec<String>
}

#[derive(StructOpt, Debug, Clone)]
pub(crate) struct BackendOptions
{
    #[structopt(long, default_value = "feoblog.sqlite3")]
    pub sqlite_file: String,
}

// Implements some functionality which may be different depending on the DB backend.
impl BackendOptions {
    fn factory_builder(&self) -> Result<Box<dyn backend::FactoryBuilder>, Error> {
        // When we support more than one kind of DB, we can switch on that here:
        Ok(
            Box::new(
                sqlite::FactoryBuilder::new(self.sqlite_file.clone())
            )
        )
    }
}

#[derive(StructOpt, Debug, Clone)]
pub(crate) enum UserCommand {
    /// List users explicitly hosted on this server.
    List(UserListCommand),

    /// Add a new user.
    Add(UserAddCommand),

    /// Remove a user
    Remove(UserRemoveCommand),
}

impl UserCommand {
    fn main(&self) -> Result<(), Error> {
        use UserCommand::*;
        match self {
            List(command) => command.main(),
            Add(command) => command.main(),
            Remove(command) => command.main(),
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
struct UserListCommand {
    #[structopt(flatten)]
    backend_options: BackendOptions,
}

impl UserListCommand {
    fn main(&self) -> Result<(), Error> {
        let factory = self.backend_options.factory_builder()?.factory()?;
        let conn = factory.open()?;
        
        conn.server_users(&mut |server_user| {

            let ServerUser{user, notes, on_homepage} = server_user;
            let on_homepage = if on_homepage { "H" } else { " " };

            println!("{} {} {}", on_homepage, user.to_base58(), notes);

            Ok(true) // fetch more
        })?;

        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
struct UserAddCommand {
    #[structopt(flatten)]
    shared_options: BackendOptions,

    user_id: UserID,

    /// Should this user's posts appear on the homepage?
    #[structopt(long)]
    on_homepage: bool,

    /// Notes for the server admin
    #[structopt(long, default_value="")]
    comment: String,
}

impl UserAddCommand {
    fn main(&self) -> Result<(), Error> {
        let factory = self.shared_options.factory_builder()?.factory()?;
        let conn = factory.open()?;

        let user = ServerUser{
            user: self.user_id.clone(),
            on_homepage: self.on_homepage,
            notes: self.comment.clone(),
        };

        conn.add_server_user(&user)?;
        Ok(())
    }
}


#[derive(StructOpt, Debug, Clone)]
struct UserRemoveCommand {
    #[structopt(flatten)]
    shared_options: BackendOptions,

    user_id: UserID,
}

impl UserRemoveCommand {
    fn main(&self) -> Result<(), Error> {
        todo!();
    }
}


#[derive(StructOpt, Debug, Clone)]
pub(crate) enum DbCommand {
    /// Initialize a new database
    Init(DbInitCommand),

    /// Upgrade an old database to the latest version.
    Upgrade(DbUpgradeCommand),

    /// Prune data from a datbase that is no longer referenced.
    Prune(DbPruneCommand),
}

impl DbCommand {
    fn main(&self) -> Result<(), Error> {
        match self {
            Self::Init(command) => command.main(),
            Self::Upgrade(command) => command.main(),
            Self::Prune(command) => command.main(),
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
struct DbInitCommand {
    #[structopt(flatten)]
    backend_options: BackendOptions,
}

impl DbInitCommand {
    fn main(&self) -> Result<(), Error> {
        let builder = self.backend_options.factory_builder()?;

        if builder.db_exists()? {
            bail!("The database already exists.");
        }

        builder.db_create()?;

        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
struct DbUpgradeCommand {
    #[structopt(flatten)]
    backend_options: BackendOptions,

    /// Verify that you've backed up your database in case this upgrade has an error.
    #[structopt(long="i-have-a-backup")]
    i_have_a_backup: bool,
}


impl DbUpgradeCommand {
    fn main(&self) -> Result<(), Error> {

        if !self.i_have_a_backup {
            println!("Please first back up your database in case there is an error during the upgrade process.");
            println!("If you have a backup, add the --i-have-a-backup option.");
            bail!("No backup");
        }

        let builder = self.backend_options.factory_builder()?;
        builder.db_upgrade()?;
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
struct DbPruneCommand {
    #[structopt(flatten)]
    backend_options: BackendOptions,

    /// Only print out statistics of what would be pruned:
    #[structopt(long)]
    dry_run: bool,

    /// Actually do the prune and delete things:
    #[structopt(long)]
    exec: bool,

    // TODO
    // blocked_users: bool,

    /// Don't delete unused attachments.
    #[structopt(long)]
    skip_unused_attachments: bool,

    /// Don't delete items belonging to unfollowed users:
    #[structopt(long)]
    skip_unfollowed_items: bool,

}

impl DbPruneCommand {
    fn main(&self) -> Result<(), Error> {
        if !self.dry_run && !self.exec {
            bail!("Must specify --dry-run or --exec");
        }

        let builder = self.backend_options.factory_builder()?;
        let conn = builder.factory()?.open()?;

        
        let result = conn.prune(PruneOpts{
            dry_run: self.dry_run,
            attachments: !self.skip_unused_attachments,
            items: !self.skip_unfollowed_items,
        })?;

        println!("{}", result);

        Ok(())
    }
}