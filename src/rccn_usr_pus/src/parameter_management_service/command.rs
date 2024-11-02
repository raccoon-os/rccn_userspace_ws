use rccn_usr::service::{CommandParseError, CommandParseResult, ServiceCommand};
use satrs::spacepackets::ecss::{tc::PusTcReader, PusPacket};
use xtce_rs::bitbuffer::BitBuffer;

pub enum Command {
    ReportParameterValues {
        number_of_parameters: u16,
        parameter_hashes: Vec<u32>,
    },

    /// Request to set a number of parameters to the new values provided in parameter_set_data.
    /// The parameter_set_data depends on the byte length of each of the parameters.
    /// Only the service handler knows this information, so we cannot extract a more
    /// meaningful data structure at parse time.
    SetParameterValues {
        number_of_parameters: u16,
        parameter_set_data: Vec<u8>,
    },
}

impl ServiceCommand for Command {
    fn from_pus_tc(tc: &PusTcReader) -> CommandParseResult<Self> {
        let buffer = &tc.app_data();
        let mut bb = BitBuffer::wrap(buffer);

        match tc.subservice() {
            1 => {
                // ReportParameterValues
                let n = bb.get_bits(16) as u16;

                let mut hashes = Vec::new();
                for _ in 0..n {
                    hashes.push(bb.get_bits(32) as u32);
                }

                Ok(Self::ReportParameterValues {
                    number_of_parameters: n,
                    parameter_hashes: hashes,
                })
            }
            2 => {
                // SetParameterValues
                let n = bb.get_bits(16) as u16;
                let data = &buffer[2..];

                Ok(Self::SetParameterValues {
                    number_of_parameters: n,
                    parameter_set_data: data.to_vec(),
                })
            }
            _ => Err(CommandParseError::UnknownSubservice(tc.subservice())),
        }
    }
}
