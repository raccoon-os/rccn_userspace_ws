use rccn_usr::service::{CommandParseError, CommandParseResult, ServiceCommand};
use satrs::spacepackets::ecss::{tc::PusTcReader, PusPacket};

pub enum ExampleServiceCommand {
    StartSdrRecording {
        center_freq_hz: u32,
        bandwidth: u32,
        duration_seconds: u16,
    },
    GenerateRandomFile {
        filename: String,
    },
}

impl ServiceCommand for ExampleServiceCommand {
    fn from_pus_tc(tc: &PusTcReader) -> CommandParseResult<Self>
    {
        match tc.subservice() {
            1 => {
                // StartSdrRecording

                // TODO parse

                Ok(Self::StartSdrRecording {
                    center_freq_hz: 2_000_000_000,
                    bandwidth: 1_000_000,
                    duration_seconds: 100,
                })
            }
            2 => {
                // GenerateRandomFile

                // TODO parse

                Ok(Self::GenerateRandomFile {
                    filename: "hello_world.bin".into(),
                })
            }
            _ => Err(CommandParseError::UnknownSubservice(tc.subservice())),
        }
    }
}
