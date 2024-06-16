mod alarm;
mod config;
mod error;
mod railway;

pub use alarm::{emit, Alarm, AlarmPayload, AlarmState};
pub use config::AlarmConfig;
pub use error::{Error, Result};
pub use railway::{
    service::{Service, Usage},
    Railway, RailwayError, RailwayResponse,
};

use chrono::{DateTime, SubsecRound, TimeDelta, Timelike, Utc};
use std::{collections::HashMap, time::Duration};
use tokio_util::sync::CancellationToken;

const MIN_PERIOD_SECS: u16 = 60;

pub async fn run() -> Result<()> {
    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();
    let shutdown_task = tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("unable to monitor ctrl+c");
        shutdown_clone.cancel();
    });

    let (railway_api_token, alarm_token, project_id, service_id) = config::required()?;

    let mut alarm_payloads: HashMap<_, _> = config::optional()?
        .into_iter()
        .map(|(alarm, config)| (alarm, AlarmPayload::from_config(config)))
        .collect();

    // Set start date to the previous minute
    let mut start_date = initial_tick(MIN_PERIOD_SECS)?;

    while let Some(alarms) = alarm::collect::alarms(
        start_date,
        &mut alarm_payloads,
        &shutdown,
        MIN_PERIOD_SECS,
        &railway_api_token,
        &project_id,
        &service_id,
    )
    .await
    {
        alarm::emit(alarms, &alarm_payloads, &alarm_token, &service_id).await;

        // Should never fail, but if it does let the process monitor restart us, this should fix the problem
        start_date = next_tick(start_date, MIN_PERIOD_SECS)?;

        // Casting like this is dangerous, but since we ensure that the min value is 0 we can trust that the i64 will fit u64 without wrapping
        let secs_to_sleep = secs_to_sleep(start_date, MIN_PERIOD_SECS);
        if secs_to_sleep > 0 {
            let sleep = tokio::time::sleep(Duration::from_secs(secs_to_sleep));
            tokio::pin!(sleep);
            tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = &mut sleep => {},
            }
        }
    }

    shutdown_task.abort();

    Ok(())
}

fn initial_tick(period_secs: u16) -> Result<DateTime<Utc>> {
    Utc::now()
        .round_subsecs(0)
        .with_second(0)
        .ok_or(Error::DateTruncation)?
        .checked_add_signed(
            TimeDelta::new(-i64::from(period_secs), 0)
                .ok_or(Error::InvalidTimeDelta(-i64::from(period_secs), 0))?,
        )
        .ok_or(Error::DateOutOfRange(Utc::now(), -i64::from(period_secs)))
}

fn next_tick(date: DateTime<Utc>, period_secs: u16) -> Result<DateTime<Utc>> {
    let date = date
        .checked_add_signed(
            TimeDelta::new(period_secs.into(), 0)
                .ok_or(Error::InvalidTimeDelta(period_secs.into(), 0))?,
        )
        .ok_or(Error::DateOutOfRange(date, period_secs.into()))?;
    Ok(date)
}

// tokio::time::interval is simpler, but it doesn't give us the date range to send to railway graphql API
fn secs_to_sleep(date: DateTime<Utc>, period_secs: u16) -> u64 {
    let secs_since_last: u64 = Utc::now().signed_duration_since(date).num_seconds().max(0) as u64;

    if secs_since_last < period_secs.into() {
        // Figures out how much to sleep until next period
        u64::from(period_secs) - secs_since_last
    } else {
        0
    }
}
