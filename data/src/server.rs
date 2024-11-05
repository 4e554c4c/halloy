use std::collections::BTreeMap;
use std::{fmt, iter, str};
use tokio::fs;
use tokio::process::Command;

use futures::channel::mpsc::Sender;
use irc::proto;
use serde::{Deserialize, Serialize};

use crate::bouncer::BouncerNetwork;
use crate::config;
use crate::config::server::Sasl;
use crate::config::Error;

pub type Handle = Sender<proto::Message>;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Server(String);

impl From<&str> for Server {
    fn from(value: &str) -> Self {
        Server(value.to_string())
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for Server {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub server: Server,
    pub config: config::Server,
    pub bouncer_network: Option<BouncerNetwork>,
}

impl Entry {
    fn primary_entry((server, v): (&Server, &MapVal)) -> Self {
        Entry {
            server: server.clone(),
            config: v.config.clone(),
            bouncer_network: None,
        }
    }
    fn from<'a>((server, v): (&'a Server, &'a MapVal)) -> impl Iterator<Item = Self> + 'a {
        iter::once(Self::primary_entry((server, v))).chain(v.children.iter().map(|network| Entry {
            server: server.clone(),
            config: v.config.clone(),
            bouncer_network: Some(network.clone()),
        }))
    }
}

#[derive(Deserialize, Clone, Debug)]
struct MapVal {
    #[serde(flatten)]
    config: config::Server,
    #[serde(skip)]
    children: Vec<BouncerNetwork>,
}

impl From<config::Server> for MapVal {
    fn from(config: config::Server) -> Self {
        Self {
            config,
            children: vec![],
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Map(BTreeMap<Server, MapVal>);

async fn read_from_command(pass_command: &str) -> Result<String, Error> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .arg("/C")
            .arg(pass_command)
            .output()
            .await?
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(pass_command)
            .output()
            .await?
    };
    if output.status.success() {
        // we remove trailing whitespace, which might be present from unix pipelines with a
        // trailing newline
        Ok(str::from_utf8(&output.stdout)?.trim_end().to_string())
    } else {
        Err(Error::ExecutePasswordCommand(String::from_utf8(
            output.stderr,
        )?))
    }
}

impl Map {
    pub fn insert(&mut self, name: Server, server: config::Server) {
        self.0.insert(name, MapVal::from(server));
    }

    pub fn insert_bouncer_network(&mut self, name: &Server, network: BouncerNetwork) {
        if let Some(entry) = self.0.get_mut(name) {
            entry.children.push(network);
        }
    }

    pub fn remove(&mut self, server: &Server) {
        self.0.remove(server);
    }

    pub fn contains(&self, server: &Server) -> bool {
        self.0.contains_key(server)
    }

    pub fn keys(&self) -> impl Iterator<Item = &Server> {
        self.0.keys()
    }

    pub fn primary_entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.entries().filter(|e| e.bouncer_network.is_none())
    }

    pub fn entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.0.iter().flat_map(Entry::from)
    }

    pub async fn read_passwords(&mut self) -> Result<(), Error> {
        for (_, MapVal { config, .. }) in self.0.iter_mut() {
            if let Some(pass_file) = &config.password_file {
                if config.password.is_some() || config.password_command.is_some() {
                    return Err(Error::DuplicatePassword);
                }
                let pass = fs::read_to_string(pass_file).await?;
                config.password = Some(pass);
            }
            if let Some(pass_command) = &config.password_command {
                if config.password.is_some() {
                    return Err(Error::DuplicatePassword);
                }
                config.password = Some(read_from_command(pass_command).await?);
            }
            if let Some(nick_pass_file) = &config.nick_password_file {
                if config.nick_password.is_some() || config.nick_password_command.is_some() {
                    return Err(Error::DuplicateNickPassword);
                }
                let nick_pass = fs::read_to_string(nick_pass_file).await?;
                config.nick_password = Some(nick_pass);
            }
            if let Some(nick_pass_command) = &config.nick_password_command {
                if config.password.is_some() {
                    return Err(Error::DuplicateNickPassword);
                }
                config.password = Some(read_from_command(nick_pass_command).await?);
            }
            if let Some(sasl) = &mut config.sasl {
                match sasl {
                    Sasl::Plain {
                        password: Some(_),
                        password_file: None,
                        password_command: None,
                        ..
                    } => {}
                    Sasl::Plain {
                        password: password @ None,
                        password_file: Some(pass_file),
                        password_command: None,
                        ..
                    } => {
                        let pass = fs::read_to_string(pass_file).await?;
                        *password = Some(pass);
                    }
                    Sasl::Plain {
                        password: password @ None,
                        password_file: None,
                        password_command: Some(pass_command),
                        ..
                    } => {
                        let pass = read_from_command(pass_command).await?;
                        *password = Some(pass);
                    }
                    Sasl::Plain { .. } => {
                        return Err(Error::DuplicateSaslPassword);
                    }
                    Sasl::External { .. } => {
                        // no passwords to read
                    }
                }
            }
        }
        Ok(())
    }
}
