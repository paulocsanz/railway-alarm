pub mod collect;
mod emit;

pub use collect::AlarmPayload;
pub use emit::emit;

use derive_get::Getters;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

#[remain::sorted]
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Hash, Display, EnumIter)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum Alarm {
    CostUpperLimit,
    CpuLowerLimitVcpus,
    CpuUpperLimitVcpus,
    DiskLowerLimitGb,
    DiskUpperLimitGb,
    EgressLowerLimitGb,
    EgressUpperLimitGb,
    HealthCheckFailed,
    IngressLowerLimitGb,
    IngressUpperLimitGb,
    MemoryLowerLimitGb,
    MemoryUpperLimitGb,
}

#[derive(Getters, Serialize, Deserialize, Clone, Debug)]
pub struct AlarmState {
    #[copy]
    alarm: Alarm,
    #[copy]
    on: bool,
}

impl AlarmState {
    pub fn new(alarm: Alarm, on: bool) -> Self {
        Self { alarm, on }
    }
}
