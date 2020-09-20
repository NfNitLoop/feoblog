#![deny(unknown_lints)]
#![deny(unused_must_use)]

#[cfg(test)]
mod tests;

use crate::backend::ServerUser;
use crate::backend::Factory;
use crate::backend::UserID;
use std::io;

use failure::{Error, bail, ResultExt};
use structopt::StructOpt;

mod backend;
mod markdown;
mod protos;
mod responder_util;
mod server;


fn main() -> Result<(), Error> {
    let command = Command::from_args();
    use Command::*;

    match command {
        Serve(command) => server::serve(command)?,
        User(command) => command.main()?,
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

    User(UserCommand)
}

#[derive(StructOpt, Debug, Clone)]

struct ServeCommand {
    #[structopt(flatten)]
    shared_options: SharedOptions,

    /// Should we open a browser window?
    #[structopt(long)]
    open: bool,

    /// Bind to this local address.
    /// If unspecified, will try to bind to some port on localhost.
    #[structopt(long="bind")]
    binds: Vec<String>
}

// TODO: Rename BackendOptions?
#[derive(StructOpt, Debug, Clone)]
pub(crate) struct SharedOptions
{
    #[structopt(default_value = "feoblog.sqlite3")]
    pub sqlite_file: String,
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
    shared_options: SharedOptions,
}

impl UserListCommand {
    fn main(&self) -> Result<(), Error> {
        let factory = backend::sqlite::Factory::new(self.shared_options.sqlite_file.clone());
        let conn = factory.open()?;
        
        conn.server_users(&mut move |server_user| {

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
    shared_options: SharedOptions,

    user_id: UserID,

    #[structopt(long)]
    on_home_page: bool,

    #[structopt(long, default_value="")]
    comment: String,
}

impl UserAddCommand {
    fn main(&self) -> Result<(), Error> {
        todo!();
    }
}


#[derive(StructOpt, Debug, Clone)]
struct UserRemoveCommand {
    #[structopt(flatten)]
    shared_options: SharedOptions,

    user_id: UserID,
}

impl UserRemoveCommand {
    fn main(&self) -> Result<(), Error> {
        todo!();
    }
}


