use crate::error::{Error, ErrorKind, ResultExt};
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io::{self, Read};
use std::path::{Path};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EngineKind {
    Eth,
    Abos
}

impl Default for EngineKind {
    fn default() -> EngineKind {
        EngineKind::Eth
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum ListenerStreamStyle {
    Vendor,
    Mapper,
}

impl Default for ListenerStreamStyle {
    fn default() -> ListenerStreamStyle {
        ListenerStreamStyle::Vendor
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
pub struct BasicItem {
    pub name: String,
    pub url: String,
    pub address: String,
    pub kind: EngineKind,
    #[serde(rename = "streamStyle")]
    pub stream_style: ListenerStreamStyle,
    pub key: Option<String>,
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
pub struct ServiceConfig {
    pub listeners: Vec<BasicItem>,
    pub senders: Vec<BasicItem>,
    #[serde(skip)]
    pub db_path: String,
    #[serde(skip)]
    pub sign_key: String,
}


impl ServiceConfig {
    pub fn from_file(file_path: &Path) -> Result<Self, Error> {
        let mut file = match fs::File::open(&file_path) {
            Ok(file) => file,
            Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
                return Err(ErrorKind::UnknownFile(format!("{:?}", file_path)).into())
            }
            Err(err) => return Err(err).chain_err(|| "Cannot open storage"),
        };

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        let config = Self::parse(&buffer)?;
        Ok(config)
    }

    pub fn parse(buf: &str) -> Result<ServiceConfig, Error> {
        // TODO, check name with unique and valid key.
        match serde_json::from_str(buf) {
            Ok(c) => Ok(c),
            Err(e) => Err(e).chain_err(|| format!("Json config parse error"))
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;
    use self::tempdir::TempDir;
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn mock_config() -> ServiceConfig{
        let items = vec![BasicItem {
            name:"kovan".to_string(),
            url: "https://kovan.infura.io/v3/838099c0002948caa607f6cfb29a4816".to_string(),
            address: "c6f47FDdB36c1442B3a7229BEE5C35e7079E2C1e".to_string(),
            kind: EngineKind::Eth,
            stream_style: ListenerStreamStyle::Vendor,
            key: None,
        }, BasicItem {
            name:"abos".to_string(),
            url: "http://47.92.173.78:1337".to_string(),
            address: "c6f47FDdB36c1442B3a7229BEE5C35e7079E2C1e".to_string(),
            kind: EngineKind::Abos,
            stream_style: ListenerStreamStyle::Mapper,
            key: Some("0x000000000000000000000".to_string()),
        }];
        let config = ServiceConfig {
            listeners: items.clone(),
            senders: items.clone(),
            sign_key: "".to_string(),
            db_path: "".into(),
        };

        let json = serde_json::to_string(&config).unwrap();
        println!("{:?}", json);
        config
    }

    fn mock_raw_data() -> &'static str {
        let data = r#"{
            "listeners": [{
                "name": "kovan",
                "url": "https://kovan.infura.io/v3/838099c0002948caa607f6cfb29a4816",
                "address": "c6f47FDdB36c1442B3a7229BEE5C35e7079E2C1e",
                "kind": "Eth",
                "streamStyle": "Vendor",
                "key": null
                }, {
                "name": "abos",
                "url": "http://47.92.173.78:1337",
                "address": "c6f47FDdB36c1442B3a7229BEE5C35e7079E2C1e",
                "kind": "Abos",
                "streamStyle": "Mapper",
                "key": "0x000000000000000000000"
                }
            ],
            "senders": [{
                "name": "kovan",
                "url": "https://kovan.infura.io/v3/838099c0002948caa607f6cfb29a4816",
                "address": "c6f47FDdB36c1442B3a7229BEE5C35e7079E2C1e",
                "kind": "Eth",
                "streamStyle": "Vendor",
                "key": null
                }, {
                "name": "abos",
                "url": "http://47.92.173.78:1337",
                "address": "c6f47FDdB36c1442B3a7229BEE5C35e7079E2C1e",
                "kind": "Abos",
                "streamStyle": "Mapper",
                "key": "0x000000000000000000000"
                }
            ]
        }
        "#;
        data
    }

    #[test]
    fn test_parse() {
        let config = ServiceConfig::parse(&mock_raw_data()).unwrap();
        let mock_config = mock_config();

        assert_eq!(config, mock_config);
    }

    #[test]
    fn test_read_from_file() {
        let config = mock_config();
        let tmp_dir = TempDir::new("example").unwrap();
        let file_path = tmp_dir.path().join("test.json");

        let mut tmp_file = File::create(&file_path).unwrap();
        tmp_file.write_all(mock_raw_data().as_bytes()).unwrap();
        tmp_file.flush().unwrap();

        let file_config = ServiceConfig::from_file(&file_path).unwrap();
        assert_eq!(config, file_config);
        drop(tmp_file);
        tmp_dir.close().unwrap();
    }
}