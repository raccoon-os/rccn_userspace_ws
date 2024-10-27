use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Result, Context, bail};

use crate::types::VcId;

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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {:?}", path.as_ref()))?;
        
        let config: Self = serde_yaml::from_str(&contents)
            .context("Failed to parse YAML config")?;
            
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        // Validate frame types
        if self.frames.r#in.frame_kind != FrameKind::Tc {
            bail!("Input frame kind must be TC");
        }
        if self.frames.out.frame_kind != FrameKind::Uslp {
            bail!("Output frame kind must be USLP");
        }

        // Validate virtual channels: check IDs are unique check ROS2 output transports
        for vc in &self.virtual_channels {
            if self.virtual_channels.iter()
                .filter(|other| other.id == vc.id)
                .count() > 1 {
                bail!("Duplicate virtual channel ID: {}", vc.id);
            }

            if let Some(OutputTransport::Ros2(t)) = &vc.out_transport {
                if t.topic_sub.is_none() && t.action_srv.is_none() {
                    bail!("Need `topic_sub` or `action_srv` for output transport of VC {}", vc.name);
                }
            }
        }

        Ok(())
    }
}
