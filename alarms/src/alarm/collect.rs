use crate::{Alarm, AlarmConfig, AlarmState, Service, Usage};
use chrono::{DateTime, Utc};
use derive_get::Getters;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

#[derive(Getters, Serialize, Deserialize, Clone, Debug)]
pub struct AlarmPayload {
    #[copy]
    accumulated: f64,
    #[copy]
    minutes: u16,
    measurements: VecDeque<bool>,
    #[copy]
    state: bool,
    config: AlarmConfig,
}

impl AlarmPayload {
    pub fn from_config(config: AlarmConfig) -> Self {
        Self {
            accumulated: 0.,
            minutes: 0,
            measurements: VecDeque::new(),
            state: false,
            config,
        }
    }
}

pub async fn alarms(
    start_date: DateTime<Utc>,
    alarm_payloads: &mut HashMap<Alarm, AlarmPayload>,
    shutdown: &CancellationToken,
    period_secs: u16,
    railway_api_token: &str,
    project_id: &str,
    service_id: &str,
) -> Option<HashMap<Alarm, AlarmState>> {
    let mut alarms = HashMap::new();

    // Tests healthcheck endpoint
    if let Some(payload) = alarm_payloads.get_mut(&Alarm::HealthCheckFailed) {
        payload.minutes += period_secs / 60;

        if payload.minutes() >= payload.config().period_minutes() {
            payload.minutes = 0;

            let is_on = tokio::select! {
                is_on = healthcheck(payload.config().value()) => is_on,
                _ = shutdown.cancelled() => return None,
            };

            process_healthcheck(&mut alarms, payload, is_on);
        }
    }

    // Gets usage or shuts-down if ctrl+c was received
    let result = tokio::select! {
        result = Service::usage(
            railway_api_token,
            project_id,
            service_id,
            start_date,
            period_secs,
        ) => result,
        _ = shutdown.cancelled() => return None,
    };

    match result {
        Ok(usage) => process_usage(&mut alarms, &mut *alarm_payloads, usage, period_secs),
        Err(err) => error!("Unable to fetch usage from Railway: {err}"),
    }

    let alarms_on = alarm_payloads
        .iter()
        .filter(|(_, payload)| payload.state())
        .map(|(alarm, _)| alarm.to_string())
        .collect::<Vec<_>>();
    if !alarms_on.is_empty() {
        info!("Alarms on: {}", alarms_on.join(", "));
    }

    Some(alarms)
}

fn process_usage(
    alarms: &mut HashMap<Alarm, AlarmState>,
    alarm_payloads: &mut HashMap<Alarm, AlarmPayload>,
    usage: Usage,
    period_secs: u16,
) {
    for (alarm, payload) in alarm_payloads {
        enum Ordering {
            Less,
            Greater,
        }

        let (measured, ordering) = match alarm {
            Alarm::CpuLowerLimitVcpus => (usage.cpu(), Ordering::Less),
            Alarm::CpuUpperLimitVcpus => (usage.cpu(), Ordering::Greater),
            Alarm::DiskLowerLimitGb => (usage.disk_gb(), Ordering::Less),
            Alarm::DiskUpperLimitGb => (usage.disk_gb(), Ordering::Greater),
            Alarm::EgressLowerLimitGb => (usage.egress_gb(), Ordering::Less),
            Alarm::EgressUpperLimitGb => (usage.egress_gb(), Ordering::Greater),
            Alarm::IngressLowerLimitGb => (usage.ingress_gb(), Ordering::Less),
            Alarm::IngressUpperLimitGb => (usage.ingress_gb(), Ordering::Greater),
            Alarm::MemoryLowerLimitGb => (usage.memory_gb(), Ordering::Less),
            Alarm::MemoryUpperLimitGb => (usage.memory_gb(), Ordering::Greater),
            Alarm::CostUpperLimit => todo!(),
            // Processed elsewhere
            Alarm::HealthCheckFailed => continue,
        };
        payload.accumulated += measured;
        payload.minutes += period_secs / 60;

        if payload.minutes() >= payload.config().period_minutes() {
            let average_measured = payload.accumulated() / f64::from(payload.minutes());
            let config_value: f64 = match payload.config().value().parse() {
                Ok(value) => value,
                Err(err) => {
                    error!("Should never happen: invalid float in {alarm} ({err})");
                    0.
                }
            };
            if config_value != 0. {
                let average_threshold = config_value / f64::from(payload.config().period_minutes());

                let alarming = match ordering {
                    Ordering::Less => average_measured < average_threshold,
                    Ordering::Greater => average_measured > average_threshold,
                };

                // Keep only the needed measurements
                payload.measurements.push_back(alarming);
                if payload.measurements().len() > payload.config().data_points().into() {
                    let _ = payload.measurements.pop_front();
                }

                // Emit alarm if enough data points alarmed
                if payload.measurements().iter().filter(|a| **a).count()
                    >= payload.config().data_points_to_alarm().into()
                {
                    if !payload.state() {
                        payload.state = true;
                        alarms.insert(*alarm, AlarmState::new(*alarm, payload.state()));
                    }
                } else if payload.state() {
                    payload.state = false;
                    alarms.insert(*alarm, AlarmState::new(*alarm, payload.state()));
                }
            }

            payload.accumulated = 0.;
            payload.minutes = 0;
        }
    }
}

async fn healthcheck(url: &str) -> bool {
    match reqwest::Client::new()
        .get(url)
        .fetch_mode_no_cors()
        .send()
        .await
    {
        Ok(response) => {
            debug!("Healthcheck {url} status {}", response.status());
            response.status() == 200
        }
        Err(err) => {
            debug!("Healthcheck {url} request failed: {err}");
            false
        }
    }
}

fn process_healthcheck(
    alarms: &mut HashMap<Alarm, AlarmState>,
    payload: &mut AlarmPayload,
    is_on: bool,
) {
    // Keep only the needed measurements
    payload.measurements.push_back(!is_on);
    if payload.measurements().len() > payload.config().data_points().into() {
        let _ = payload.measurements.pop_front();
    }

    // Emit alarm if enough data points alarmed
    if payload.measurements().iter().filter(|a| **a).count()
        >= payload.config().data_points_to_alarm().into()
    {
        if !payload.state() {
            payload.state = true;
            alarms.insert(
                Alarm::HealthCheckFailed,
                AlarmState::new(Alarm::HealthCheckFailed, payload.state()),
            );
        }
    } else if payload.state() {
        payload.state = false;
        alarms.insert(
            Alarm::HealthCheckFailed,
            AlarmState::new(Alarm::HealthCheckFailed, payload.state()),
        );
    }
}
