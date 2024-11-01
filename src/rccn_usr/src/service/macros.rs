#[macro_export]
macro_rules! impl_verification_sender {
    ($name:ident, $state_in:ty, $state_out:ty) => {
        paste::paste! {
            pub fn [<send_ $name _success>](
                &self,
                token: VerificationToken<$state_in>,
            ) -> Result<VerificationToken<$state_out>, EcssTmtcError> {
                let tm_sender = self.get_default_tm_sender();
                let reporter = self.verification_reporter.clone();
                let timestamp = self.timestamp_helper.stamp();

                reporter.[<$name _success>](&tm_sender, token, timestamp)
            }

            pub fn [<send_ $name _failure>](
                &self,
                token: VerificationToken<$state_in>,
                failure_code: &dyn EcssEnumeration,
                failure_data: &[u8],
            ) -> Result<(), EcssTmtcError> {
                let tm_sender = self.get_default_tm_sender();
                let reporter = self.verification_reporter.clone();
                let timestamp = self.timestamp_helper.stamp();

                reporter.[<$name _failure>](
                    &tm_sender,
                    token,
                    FailParams::new(timestamp, failure_code, failure_data),
                )
            }
        }
    };
}
