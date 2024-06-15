mod alarm;
mod config;
mod error;
mod railway;

pub use alarm::{emit, Alarm};
pub use config::AlarmConfig;
pub use error::{Error, Result};
pub use railway::{
    service::{Service, Usage},
    Railway, RailwayError, RailwayResponse,
};

use chrono::{TimeDelta, Utc};
use derive_get::Getters;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap, collections::VecDeque, sync::atomic::AtomicBool, sync::atomic::Ordering,
    sync::Arc, time::Duration,
};
use tracing::error;

#[derive(Getters, Serialize, Deserialize, Clone, Debug)]
pub struct AlarmState {
    alarm: Alarm,
    on: bool,
}

impl AlarmState {
    pub fn new(alarm: Alarm, on: bool) -> Self {
        Self { alarm, on }
    }
}

#[derive(Getters, Serialize, Deserialize, Clone, Debug)]
pub struct AlarmPayload {
    #[copy]
    accumulated: f64,
    #[copy]
    minutes: u16,
    measurements: VecDeque<bool>,
    #[copy]
    state: bool,
    #[copy]
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

const MIN_PERIOD_SECS: u16 = 60;

pub async fn run() -> Result<()> {
    // Shutdown handler
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);
    ctrlc::set_handler(move || shutdown_clone.store(true, Ordering::Relaxed))?;

    let (railway_api_token, alarm_token, project_id, service_id) = required_env_vars()?;

    let mut alarm_payloads: HashMap<_, _> = config::get()?
        .into_iter()
        .map(|(alarm, config)| (alarm, AlarmPayload::from_config(config)))
        .collect();

    let mut start_date = Utc::now();
    while shutdown.load(Ordering::Relaxed) {
        let mut alarms = HashMap::new();

        match Service::usage(
            &railway_api_token,
            &project_id,
            &service_id,
            start_date,
            MIN_PERIOD_SECS,
        )
        .await
        {
            Ok(usage) => {
                for (alarm, payload) in &mut alarm_payloads {
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
                        Alarm::HealthCheckFailed => continue,
                        Alarm::CostLowerLimit => continue,
                        Alarm::CostUpperLimit => continue,
                    };
                    payload.accumulated += measured;
                    payload.minutes += MIN_PERIOD_SECS / 60;

                    if payload.minutes() >= payload.config().period_minutes() {
                        let average_measured = payload.accumulated() / f64::from(payload.minutes());
                        let average_threshold =
                            payload.config().value() / f64::from(payload.config().period_minutes());

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
                        if payload.measurements().iter().copied().count()
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

                        payload.accumulated = 0.;
                        payload.minutes = 0;
                    }
                }
            }
            Err(err) => {
                error!("Unable to fetch usage from Railway: {err}");
            }
        }

        // Gets other active alarms to send with the one that changed
        if !alarms.is_empty() {
            for (alarm, payload) in &alarm_payloads {
                alarms.insert(*alarm, AlarmState::new(*alarm, payload.state()));
            }
        }

        alarm::emit(
            alarms.into_iter().map(|(_, v)| v).collect(),
            &alarm_token,
            &service_id,
        )
        .await;

        // Casting like this is dangerous, but since we ensure that the min value is 0 we can trust that the i64 will fit u64 without wrapping
        let secs_since_last: u64 = Utc::now()
            .signed_duration_since(start_date)
            .num_seconds()
            .max(0) as u64;
        if secs_since_last < MIN_PERIOD_SECS.into() {
            // Figure out how much to sleep for the interval
            let secs_to_sleep = u64::from(MIN_PERIOD_SECS) - secs_since_last;
            tokio::time::sleep(Duration::from_secs(secs_to_sleep)).await;
        }

        // Get next minute
        // Should never fail, but if it does let the process monitor restart us, this should fix the problem
        start_date = start_date
            .checked_add_signed(
                TimeDelta::new(MIN_PERIOD_SECS.into(), 0)
                    .ok_or(Error::InvalidTimeDelta(MIN_PERIOD_SECS.into(), 0))?,
            )
            .ok_or(Error::DateOutOfRange(start_date, MIN_PERIOD_SECS.into()))?;
    }

    Ok(())
}

fn required_env_vars() -> Result<(String, String, String, String)> {
    let railway_api_token = std::env::var("RAILWAY_API_TOKEN")
        .map_err(|_| Error::MissingEnvVar("RAILWAY_API_TOKEN"))?;
    let alarm_token =
        std::env::var("ALARM_TOKEN").map_err(|_| Error::MissingEnvVar("ALARM_TOKEN"))?;

    let project_id = std::env::var("RAILWAY_PROJECT_ID")
        .map_err(|_| Error::MissingEnvVar("RAILWAY_PROJECT_ID"))?;
    let service_id = std::env::var("RAILWAY_SERVICE_ID")
        .map_err(|_| Error::MissingEnvVar("RAILWAY_SERVICE_ID"))?;
    Ok((railway_api_token, alarm_token, project_id, service_id))
}
