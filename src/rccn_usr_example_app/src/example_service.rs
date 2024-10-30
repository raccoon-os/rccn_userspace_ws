use rccn_usr::{
    service::{
        CommandParseError, CommandParseResult, PusService, PusServiceBase, ServiceCommand,
        ServiceResult,
    },
    types::VirtualChannelTxMap,
};
use satrs::{
    pus::verification::TcStateAccepted,
    pus::verification::VerificationToken,
    spacepackets::ecss::{tc::PusTcReader, PusPacket},
};

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
    where
        Self: Sized,
    {
        match tc.subservice() {
            1 => {
                // StartSdrRecording

                Ok(Self::StartSdrRecording {
                    center_freq_hz: 2_000_000_000,
                    bandwidth: 1_000_000,
                    duration_seconds: 100,
                })
            }
            2 => {
                // GenerateRandomFile

                Ok(Self::GenerateRandomFile {
                    filename: "hello_world.bin".into(),
                })
            }
            _ => Err(CommandParseError::UnknownSubservice(tc.subservice())),
        }
    }
}

pub struct ExampleService {
    service_base: PusServiceBase,
}

impl ExampleService {
    pub fn new(apid: u16, vc_map: &VirtualChannelTxMap) -> Self {
        Self {
            service_base: PusServiceBase::new(apid, 142, 12345u64, vc_map),
        }
    }
}

impl PusService for ExampleService {
    type CommandT = ExampleServiceCommand;

    fn get_service_base(&mut self) -> &mut PusServiceBase {
        &mut self.service_base
    }

    fn handle_tc(
        &self,
        tc: &Self::CommandT,
        _token: VerificationToken<TcStateAccepted>,
    ) -> ServiceResult<()> {
        match tc {
            ExampleServiceCommand::StartSdrRecording {
                center_freq_hz,
                bandwidth,
                duration_seconds,
            } => {
                println!("SDR magic goes here {center_freq_hz} {bandwidth} {duration_seconds}");
            }
            ExampleServiceCommand::GenerateRandomFile { filename } => {
                println!("Write random data to {filename}");
            }
        }

        Ok(())
    }
}
