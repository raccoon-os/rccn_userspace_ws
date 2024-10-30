use std::sync::{Arc, Mutex};

use crate::{
    time::TimestampHelper,
    types::{RccnEcssTmSender, VirtualChannelTxMap},
};
use satrs::{
    pus::verification::TcStateAccepted,
    spacepackets::ecss::{tc::PusTcReader, PusError, PusPacket},
};
use satrs::{
    pus::{
        verification::{
            FailParams, TcStateNone, VerificationReporter, VerificationReporterCfg,
            VerificationReportingProvider, VerificationToken,
        },
        EcssTmtcError,
    },
    spacepackets::ecss::{EcssEnumU8, EcssEnumeration},
    ComponentId,
};

pub enum PusServiceError {
    PusError(PusError),
}

pub type ServiceResult<T> = Result<T, PusServiceError>;

#[derive(Clone)]
pub struct PusServiceBase {
    pub apid: u16,
    pub service: u8,
    pub verification_reporter: VerificationReporter,
    pub virtual_channel_tx: VirtualChannelTxMap,
    pub timestamp_helper: TimestampHelper,
    pub component_id: ComponentId,
    pub msg_counter: Arc<Mutex<u16>>,
}

impl PusServiceBase {
    pub fn new(
        apid: u16,
        service: u8,
        component_id: ComponentId,
        vc_map: &VirtualChannelTxMap,
    ) -> Self {
        let verification_reporter_cfg =
            VerificationReporterCfg::new(apid, 1, 1, 100).expect("Invalid APID");

        Self {
            apid,
            service,
            virtual_channel_tx: vc_map.clone(),
            timestamp_helper: TimestampHelper::new(),
            verification_reporter: VerificationReporter::new(
                component_id,
                &verification_reporter_cfg,
            ),
            component_id,
            msg_counter: Arc::new(Mutex::new(0u16)),
        }
    }

    pub fn get_default_tm_sender(&self) -> RccnEcssTmSender {
        let tm_channel_tx = self
            .virtual_channel_tx
            .get(&0)
            .expect("Could not get TM sender for VC 0");

        RccnEcssTmSender {
            channel: tm_channel_tx.clone(),
            msg_counter: self.msg_counter.clone(),
        }
    }

    pub fn send_acceptance_failure(
        &self,
        token: VerificationToken<TcStateNone>,
        failure_code: &dyn EcssEnumeration,
        failure_data: &[u8],
    ) -> Result<(), EcssTmtcError> {
        let tm_sender = self.get_default_tm_sender();
        let reporter = self.verification_reporter.clone();

        reporter.acceptance_failure(
            &tm_sender,
            token,
            FailParams::new(self.timestamp_helper.stamp(), failure_code, failure_data),
        )
    }
}

#[derive(Debug, Clone)]
pub enum CommandParseError {
    UnknownSubservice(u8),
    Other,
}

pub type CommandParseResult<T> = Result<T, CommandParseError>;

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum AcceptanceError {
    PusError(PusError) = 1,
    UnknownApid(u16),
    UnknownService(u8),
    UnknownSubservice(u8),
    CommandParseError(CommandParseError),
    ArgumentError,
    ServiceDisconnected,
    SendVerificationTmFailed
}

impl Into<EcssEnumU8> for AcceptanceError {
    fn into(self) -> EcssEnumU8 {
        let tag = match self {
            AcceptanceError::PusError(_) => 1,
            AcceptanceError::UnknownApid(_) => 2,
            AcceptanceError::UnknownService(_) => 3,
            AcceptanceError::UnknownSubservice(_) => 4,
            AcceptanceError::CommandParseError(_) => 5,
            AcceptanceError::ArgumentError => 6,
            AcceptanceError::ServiceDisconnected => 7,
            AcceptanceError::SendVerificationTmFailed => 8
        };

        EcssEnumU8::new(tag)
    }
}

type AcceptanceResult = Result<(), AcceptanceError>;

pub trait ServiceCommand {
    fn from_pus_tc(tc: &PusTcReader) -> CommandParseResult<Self>
    where
        Self: Sized;
}

pub trait PusService {
    type CommandT: ServiceCommand;

    fn get_service_base(&mut self) -> &mut PusServiceBase;

    fn handle_tc(
        &self,
        tc: &Self::CommandT,
        token: VerificationToken<TcStateAccepted>,
    ) -> ServiceResult<()>;

    fn handle_tc_bytes(&mut self, bytes: &[u8]) -> AcceptanceResult {
        let base = self.get_service_base();

        // ST[01] verification util
        let mut reporter = base.verification_reporter.clone();

        let pus_tc = match PusTcReader::new(&bytes) {
            Err(e) => {
                // Could not parse the incoming bytes as a PUS TC.
                // Since we don't know what we received, we also shouldn't send an error back.
                Err(AcceptanceError::PusError(e))
            }
            Ok((tc, _size)) => Ok(tc),
        }?;

        let token = reporter.add_tc(&pus_tc);
        base.timestamp_helper.update_from_now();

        // Check if the TC is destined for this service
        if pus_tc.service() != base.service {
            return Err(AcceptanceError::UnknownService(pus_tc.service()));
        }

        // Try to parse the TC using the service's CommandT
        // Send an acceptance failure TM frame if we couldn't parse the TC
        let app_tc = Self::CommandT::from_pus_tc(&pus_tc).map_err(|parse_error| {
            let err = AcceptanceError::CommandParseError(parse_error);

            // Send acceptance failure
            let err_code: EcssEnumU8 = err.clone().into();
            let send_result = base.send_acceptance_failure(token, &err_code, &[]);
            if let Err(e) = send_result {
                println!("Error sending acceptance failure TM: {e}");
            };

            err
        })?;

        base.timestamp_helper.update_from_now();

        // Send TC accepted telemetry, get TC accepted token
        let accepted_token = reporter
            .acceptance_success(
                &base.get_default_tm_sender(),
                token,
                base.timestamp_helper.stamp(),
            )
            .map_err(|err| {
                println!("Error sending acceptance success telemetry: {err}");
                AcceptanceError::SendVerificationTmFailed
            })?;

        let _ = self.handle_tc(&app_tc, accepted_token);

        Ok(())
    }
}
