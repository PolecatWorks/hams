use std::{fmt::Display, ops::Deref};

use crate::health::HealthProbeResult;
use serde::Serialize;

/// reply for a health check summarising a vec of probes
pub struct HealthCheckResults<'a>(pub Vec<HealthProbeResult<'a>>);

impl<'a> HealthCheckResults<'a> {
    /// Check all probes are true then this check returns true
    pub fn valid(&self) -> bool {
        self.iter().all(|probe_result| probe_result.valid)
    }
}

impl<'a> Deref for HealthCheckResults<'a> {
    type Target = Vec<HealthProbeResult<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<HealthCheckResults<'a>> for Vec<HealthProbeResult<'a>> {
    fn from(val: HealthCheckResults<'a>) -> Self {
        val.0
    }
}

impl<'a> Display for HealthCheckResults<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;

        let mut first = true;

        self.iter().try_fold((), |result, check| {
            if first {
                first = false;
                write!(f, "{}", check)
            } else {
                write!(f, ",{}", check)
            }
        })?;

        write!(f, "]")?;
        Ok(())
    }
}

/// Result from whole HealthSystem and can be ready for sending as Http Reply
#[derive(Debug, Serialize)]
pub struct HealthCheckReply<'a> {
    pub(crate) name: &'a str,
    pub(crate) valid: bool,
    pub(crate) detail: Vec<HealthProbeResult<'a>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_of_probe() {
        let probe_result = HealthProbeResult {
            name: "probe0",
            valid: true,
        };

        assert_eq!("probe0/true", probe_result.to_string());
    }

    #[test]
    fn display_of_check() {
        let check_result_empty = HealthCheckResults(vec![]);
        assert_eq!("[]", check_result_empty.to_string());

        let check_result_single = HealthCheckResults(vec![HealthProbeResult {
            name: "probe0",
            valid: true,
        }]);
        assert_eq!("[probe0/true]", check_result_single.to_string());

        let check_result_empty = HealthCheckResults(vec![
            HealthProbeResult {
                name: "probe0",
                valid: true,
            },
            HealthProbeResult {
                name: "probe1",
                valid: true,
            },
        ]);
        assert_eq!("[probe0/true,probe1/true]", check_result_empty.to_string());
    }
}
