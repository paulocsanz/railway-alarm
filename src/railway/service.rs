use crate::{Error, Railway, Result};
use chrono::{DateTime, TimeDelta, Utc};
use derive_get::Getters;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use tracing::warn;

const USAGE: &str = include_str!("../graphql/usage.gql");

#[remain::sorted]
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Hash, Display, EnumIter)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Alarm {
    CostLowerLimit,
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

#[derive(Getters, Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    cpu: f64,
    memory_gb: f64,
    disk_gb: f64,
    ingress_gb: f64,
    egress_gb: f64,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

pub struct Service;

impl Service {
    pub async fn usage(
        token: &str,
        project_id: &str,
        service_id: &str,
        start_date: DateTime<Utc>,
        period_secs: u16,
    ) -> Result<Usage> {
        let end_date = start_date
            .checked_add_signed(
                TimeDelta::new(period_secs.into(), 0)
                    .ok_or(Error::InvalidTimeDelta(period_secs.into(), 0))?,
            )
            .ok_or(Error::DateOutOfRange(start_date, period_secs.into()))?;
        let response: UsageResponse = Railway::query(
            token,
            serde_json::json!({
                "query": USAGE,
                "variables": {
                    "projectId": project_id,
                    "startDate": start_date,
                    "endDate": end_date,
                },
            }),
        )
        .await?;

        #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
        #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
        pub enum MeasurementResponse {
            CpuUsage,
            MemoryUsageGb,
            DiskUsageGb,
            NetworkRxGb,
            NetworkTxGb,
        }

        #[derive(Serialize, Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct TagsResponse {
            service_id: Option<String>,
        }

        #[derive(Serialize, Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct IndividualUsage {
            measurement: MeasurementResponse,
            tags: TagsResponse,
            value: f64,
        }

        #[derive(Serialize, Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct UsageResponse {
            usage: Vec<IndividualUsage>,
        }

        let mut cpu = None;
        let mut memory_gb = None;
        let mut disk_gb = None;
        let mut ingress_gb = None;
        let mut egress_gb = None;

        let mut any = false;
        for usage in response.usage {
            if usage.tags.service_id.as_deref() == Some(service_id) {
                any = true;
                match usage.measurement {
                    MeasurementResponse::CpuUsage => cpu = Some(usage.value),
                    MeasurementResponse::MemoryUsageGb => memory_gb = Some(usage.value),
                    MeasurementResponse::DiskUsageGb => disk_gb = Some(usage.value),
                    MeasurementResponse::NetworkRxGb => ingress_gb = Some(usage.value),
                    MeasurementResponse::NetworkTxGb => egress_gb = Some(usage.value),
                }
            }
        }

        if !any {
            warn!("No measurements collected for service {service_id}");
        }

        Ok(Usage {
            cpu: cpu.unwrap_or(0.),
            memory_gb: memory_gb.unwrap_or(0.),
            disk_gb: disk_gb.unwrap_or(0.),
            ingress_gb: ingress_gb.unwrap_or(0.),
            egress_gb: egress_gb.unwrap_or(0.),
            start_date,
            end_date,
        })
    }
}
