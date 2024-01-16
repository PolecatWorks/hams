/// The [HealthProbeInner] trait defines what must be defined by the health probe for it to be used by Hams
/// [HpW] takes the [HealthProbeInner] and wraps it behind an Arc Mutex to allow it to be used
/// effectively as a HealthCheck (ie it has a role at the code site AND inside the [HealthCheck])
///
/// [HpW] provides an upper interface called [HealthProbe] which allows the HpW to be used within
/// a HashSet in the [HealthCheck].
///
/// [HealthProbeWrapper] is EOL and to be removed soon.

/// Plans:
/// Add an FFI safe interface for a [HealthProbeInner] to allow externally created [HealthProbeInner]s to be transferred
/// into [Hams]. At this point the [HealthProbeInner] is transferred across.
///
/// Questions:
/// If we transfer in a Probe into Hams then Hams will expect to access the object. The object needs
/// to maintain its own safety (eg Arc and Mutex). SO this seems wrong. ie our HealthProbeInner seems the wrong interface to be exposing to FFI.
///
/// Looking at [HealthProbe]. This provides a limited a HashSet capable dyn interface to the [HealthCheck].
/// Consider can we create the FFI interface for Probes around this interface.
///
/// If we look at the [HealthProbe] this could be replaced by the FFI safe interface and would result in an object that is
/// HashSet viable. Care would need to be taken to ensure lifetimes of unsafe objects was preserved. (eg owndedi interface)
/// The [HealthProbe] does not have an opinion on the
pub mod health_check;
pub mod health_probe;
pub mod health_result;
pub mod kick;
pub mod manual;
pub mod health_probe2;


pub use health_check::{HealthCheck, HealthCheckReply};
pub use health_probe::{HealthProbeInner, HealthProbeResult, HealthProbeWrapper};
pub use health_result::HealthCheckResults;
pub use kick::Kick;
pub use manual::Manual;
pub use health_probe2::Poisoned;
