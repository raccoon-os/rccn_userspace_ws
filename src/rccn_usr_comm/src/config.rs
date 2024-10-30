use serde::{Deserialize, Serialize};
use std::{path::Path, io};
use thiserror::Error;
use rccn_usr::types::VcId;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Config validation error: {0}")]
    Validation(String),
}

macro_rules! config_structs {
    ($($item:item)*) => {
        $(
            #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
            $item
        )*
    };
}

macro_rules! config_enums {
    ($($item:item)*) => {
        $(
            #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
            #[serde(tag = "kind", rename_all = "lowercase")]
            $item
        )*
    };
}

config_structs! {
    pub struct UdpInputTransport {
        pub listen: String
    }

    pub struct UdpOutputTransport {
        pub send: String
    }

    pub struct Ros2InputTransport {
        pub topic_pub: String
    }

    pub struct Ros2OutputTransport {
        pub topic_sub: Option<String>,
        pub action_srv: Option<String>
    }

    pub struct FrameConfig {
        pub frame_kind: FrameKind,
        pub transport: InputTransport
    }

    pub struct FrameOutConfig {
        pub frame_kind: FrameKind,
        pub transport: OutputTransport
    }

    pub struct VirtualChannel {
        pub id: VcId,
        pub name: String,
        pub splitter: Option<String>,
        pub in_transport: Option<InputTransport>,
        pub out_transport: Option<OutputTransport>
    }

    pub struct Frames {
        pub spacecraft_id: u16,
        pub r#in: FrameConfig,
        pub out: FrameOutConfig
    }

    pub struct Config {
        pub frames: Frames,
        pub virtual_channels: Vec<VirtualChannel>
    }
}

config_enums! {
    pub enum InputTransport {
        Udp(UdpInputTransport),
        Ros2(Ros2InputTransport)
    }

    pub enum OutputTransport {
        Udp(UdpOutputTransport),
        Ros2(Ros2OutputTransport)
    }
}

// Special case, we need Copy for FrameKind ... and I'm not good at rust macros
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
#[serde(rename_all = "lowercase")]
pub enum FrameKind {
    Tc,
    Uslp,
}


impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(&path)?;
        let config: Self = serde_yaml::from_str(&contents)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        // Validate frame types
        if self.frames.r#in.frame_kind != FrameKind::Tc {
            return Err(ConfigError::Validation("Input frame kind must be TC".into()));
        }
        if self.frames.out.frame_kind != FrameKind::Uslp {
            return Err(ConfigError::Validation("Output frame kind must be USLP".into()));
        }

        // Validate virtual channels: check IDs are unique check ROS2 output transports
        for vc in &self.virtual_channels {
            if self.virtual_channels.iter()
                .filter(|other| other.id == vc.id)
                .count() > 1 {
                return Err(ConfigError::Validation(format!("Duplicate virtual channel ID: {}", vc.id)));
            }

            if let Some(OutputTransport::Ros2(t)) = &vc.out_transport {
                if t.topic_sub.is_none() && t.action_srv.is_none() {
                    return Err(ConfigError::Validation(
                        format!("Need `topic_sub` or `action_srv` for output transport of VC {}", vc.name)
                    ));
                }
            }
        }

        Ok(())
    }
}
