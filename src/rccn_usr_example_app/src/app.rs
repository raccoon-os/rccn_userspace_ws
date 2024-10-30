use rccn_usr::{time::TimestampHelper, types::VirtualChannelOutMap};
use satrs::pus::verification::VerificationReporter;
use spacepackets::{
    ecss::{
        tc::PusTcReader,
        tm::{PusTmCreator, PusTmSecondaryHeader},
        PusError, WritablePusPacket,
    },
    PacketId, PacketSequenceCtrl, PacketType, SequenceFlags, SpHeader,
};

pub enum PusServiceError {
    PusError(PusError),
}

type ServiceResult<T> = Result<T, PusServiceError>;

#[derive(Clone)]
pub struct PusServiceCommon {
    pub apid: u16,
    pub verification_reporter: VerificationReporter,
    pub virtual_channel_tx: VirtualChannelOutMap,
    pub timestamp_helper: TimestampHelper
}

impl PusServiceCommon {
    fn handle_tc_bytes(&self, bytes: Vec<u8>) -> ServiceResult<()> {
        let tc = match PusTcReader::new(&bytes) {
            Err(e) => {
                // TODO send error telemetry
                Err(PusServiceError::PusError(e))
            },
            Ok((tc, _size)) => Ok(tc)
        }?;

        // Check if the TC is destined for this application

        Ok(())
    }
}
