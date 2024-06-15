use crate::{Alarm, Result};
use derive_get::Getters;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use tracing::warn;

#[derive(Getters, Serialize, Deserialize, Copy, Clone, Debug)]
pub struct AlarmConfig {
    #[copy]
    value: f64,
    #[copy]
    period_minutes: u16,
    #[copy]
    data_points: u16,
    #[copy]
    data_points_to_alarm: u16,
}

const DEFAULT_PERIOD_MINUTES: u16 = 1;
const MIN_PERIOD_MINUTES: u16 = 1;

const DEFUALT_DATA_POINTS: u16 = 5;
const MIN_DATA_POINTS: u16 = 1;

const DEFUALT_DATA_POINTS_TO_ALARM: u16 = 3;
const MIN_DATA_POINTS_TO_ALARM: u16 = 1;

pub fn get() -> Result<HashMap<Alarm, AlarmConfig>> {
    let default_period_minutes = std::env::var("PERIOD_MINUTES")
        .ok()
        .map(|value| value.parse::<u16>())
        .transpose()?
        .unwrap_or(DEFAULT_PERIOD_MINUTES);
    let default_data_points = std::env::var("DATA_POINTS")
        .ok()
        .map(|value| value.parse::<u16>())
        .transpose()?
        .unwrap_or(DEFUALT_DATA_POINTS);
    let default_data_points_to_alarm = std::env::var("DATA_POINTS_TO_ALARM")
        .ok()
        .map(|value| value.parse::<u16>())
        .transpose()?
        .unwrap_or(DEFUALT_DATA_POINTS_TO_ALARM);

    let mut configs = HashMap::new();
    for alarm in Alarm::iter() {
        if let Some(value) = std::env::var(dbg!(alarm.to_string()))
            .ok()
            .map(|value| value.parse::<f64>())
            .transpose()?
        {
            if value != 0. {
                let period_minutes_env_name = dbg!(format!("{alarm}_PERIOD_MINUTES"));
                let mut period_minutes = std::env::var(&period_minutes_env_name)
                    .ok()
                    .map(|value| value.parse::<u16>())
                    .transpose()?
                    .unwrap_or(default_period_minutes);
                if period_minutes < MIN_PERIOD_MINUTES {
                    period_minutes = MIN_PERIOD_MINUTES;
                    warn!("{period_minutes_env_name} can't be below {MIN_PERIOD_MINUTES}, setting it to {MIN_PERIOD_MINUTES}");
                }

                let data_points_env_name = dbg!(format!("{alarm}_DATA_POINTS"));
                let mut data_points = std::env::var(&data_points_env_name)
                    .ok()
                    .map(|value| value.parse::<u16>())
                    .transpose()?
                    .unwrap_or(default_data_points);
                if data_points < MIN_DATA_POINTS {
                    data_points = MIN_DATA_POINTS;
                    warn!("{data_points_env_name} can't be below {MIN_DATA_POINTS}, setting it to {MIN_DATA_POINTS}");
                }

                let data_points_to_alarm_env_name = dbg!(format!("{alarm}_DATA_POINTS_TO_ALARM"));
                let mut data_points_to_alarm = std::env::var(&data_points_to_alarm_env_name)
                    .ok()
                    .map(|value| value.parse::<u16>())
                    .transpose()?
                    .unwrap_or(default_data_points_to_alarm);
                if data_points_to_alarm < MIN_DATA_POINTS_TO_ALARM {
                    data_points_to_alarm = MIN_DATA_POINTS_TO_ALARM;
                    warn!("{data_points_to_alarm_env_name} can't be below {MIN_DATA_POINTS_TO_ALARM}, setting it to {MIN_DATA_POINTS_TO_ALARM}");
                }

                configs.insert(
                    alarm,
                    AlarmConfig {
                        value,
                        period_minutes,
                        data_points,
                        data_points_to_alarm,
                    },
                );
            }
        }
    }
    Ok(configs)
}
