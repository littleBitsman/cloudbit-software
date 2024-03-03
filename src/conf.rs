extern crate toml;
extern crate serde;
extern crate serde_json;

pub mod cloud_config {
    use std::{collections::HashMap, fs};

    use serde::{Deserialize, Deserializer};

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub struct CloudClientConfig {
        pub cloud_url: String,
        pub mac_address: String,
        pub cb_id: String
    }

    impl<'de> Deserialize<'de> for CloudClientConfig {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let map: HashMap<String, serde_json::Value> = HashMap::deserialize(deserializer)?;
            
            let mac_address = fs::read_to_string("/var/lb/mac").unwrap();
            let cb_id = fs::read_to_string("/var/lb/id").unwrap();

            Ok(CloudClientConfig {
                cloud_url: map["cloud_parameters"]["cloud_url"].as_str().unwrap().to_string(),
                mac_address,
                cb_id,
            })
        }
    }

    #[allow(dead_code)]
    pub fn parse(path: &str) -> Result<CloudClientConfig, String> {
        let read = fs::read(path);
        if read.is_err() {
            return Err(format!("{:?}", read.unwrap_err()));
        };
        
        let parsed: Result<CloudClientConfig, toml::de::Error> = toml::de::from_str(&std::str::from_utf8(&read.unwrap()).unwrap());
        if parsed.is_err() {
            return Err(format!("{:?}", parsed.unwrap_err()));
        };

        Ok(parsed.unwrap())
    }
}
