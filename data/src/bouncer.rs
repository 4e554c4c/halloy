use std::collections::BTreeMap;

use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NetworkState {
    Connected,
    Connecting,
    Disconnected,
}

impl FromStr for NetworkState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "connected" => Ok(Self::Connected),
            "connecting" => Ok(Self::Connecting),
            "disconnected" => Ok(Self::Disconnected),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BouncerNetwork {
    pub id: String,
    pub name: String,
    pub host: String,
    pub state: NetworkState,
    pub port: Option<u16>,
    pub tls: Option<bool>,
    pub pass: Option<String>,
    pub nickname: Option<String>,
    pub realname: Option<String>,
    pub error: Option<String>,
}

impl BouncerNetwork {
    pub fn parse(id: String, s: &str) -> Option<Self> {
        let parameter_map: BTreeMap<_, _> =
            s.split(';').map(|k| k.split_once('=')).flatten().collect();

        Some(BouncerNetwork {
            id,
            name: parameter_map.get("name")?.to_string(),
            host: parameter_map.get("host")?.to_string(),
            port: parameter_map.get("port").and_then(|s| s.parse().ok()),
            nickname: parameter_map.get("nickname").map(|s| s.to_string()),
            realname: parameter_map.get("realname").map(|s| s.to_string()),
            pass: parameter_map.get("pass").map(|s| s.to_string()),
            state: parameter_map.get("port")?.parse().ok()?,
            tls: match parameter_map.get("port").map(|s| *s) {
                Some("1") => Some(true),
                Some("0") => Some(false),
                _ => None,
            },
            error: parameter_map.get("error").map(|s| s.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
