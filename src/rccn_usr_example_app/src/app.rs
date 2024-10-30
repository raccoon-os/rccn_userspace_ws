use rccn_usr::{time::TimestampHelper, types::VirtualChannelOutMap};
use satrs::pus::verification::VerificationReporter;
use spacepackets::
    ecss::{
        tc::PusTcReader,
        PusError,
    }
;

pub enum PusServiceError {
    PusError(PusError),
}

type ServiceResult<T> = Result<T, PusServiceError>;

#[derive(Clone)]
pub struct PusServiceCommon<'a> {
    pub apid: u16,
    pub verification_reporter: VerificationReporter,
    pub virtual_channel_tx: &'a VirtualChannelOutMap,
    pub timestamp_helper: TimestampHelper
}


pub enum CommandParseError {
    Unknown()
}

type CommandParseResult<T> = Result<T, CommandParseError>;

pub enum AcceptanceError {
    PusError(PusError),
    UnknownApid(u16),
    UnknownService(u16),
    UnknownSubservice(u16),
    CommandParseError(CommandParseError),
    ArgumentError(),
    ServiceDisconnected()
}

type AcceptanceResult = Result<(), AcceptanceError>;

pub trait ServiceCommand {
    fn from_pus_tc(tc: &PusTcReader) -> CommandParseResult<Self> where Self: Sized;
}

pub trait PusService {
    type CommandT: ServiceCommand;

    fn get_service_common(&self) -> &mut PusServiceCommon;

    fn handle_tc(&self, tc: &Self::CommandT) -> ServiceResult<()>;

    fn handle_tc_bytes(&self, bytes: &[u8]) -> AcceptanceResult {
        let common = self.get_service_common();

        // TODO configure which TM channel to transmit on
        let tm_sender = common.virtual_channel_tx.get(&0);

        let pus_tc = match PusTcReader::new(&bytes) {
            Err(e) => {
                // TODO send error telemetry
                //common.verification_reporter.acceptance_failure(sender, token, params)
                Err(AcceptanceError::PusError(e))
            },
            Ok((tc, _size)) => Ok(tc)
        }?;

        let app_tc = match Self::CommandT::from_pus_tc(&pus_tc) {
            Err(_) => todo!(),
            Ok(tc) => Ok(tc)
        }?;

        // Check if the TC is destined for this application

        Ok(())
    }
}