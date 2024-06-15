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

const DEFAULT_DATA_POINTS: u16 = 5;
const MIN_DATA_POINTS: u16 = 1;

const DEFAULT_DATA_POINTS_TO_ALARM: u16 = 3;
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
        .unwrap_or(DEFAULT_DATA_POINTS);
    let default_data_points_to_alarm = std::env::var("DATA_POINTS_TO_ALARM")
        .ok()
        .map(|value| value.parse::<u16>())
        .transpose()?
        .unwrap_or(DEFAULT_DATA_POINTS_TO_ALARM);

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

#[cfg(test)]
mod tests {
    use crate::Alarm;
    use strum::IntoEnumIterator;

    #[test]
    fn all() {
        // All
        for alarm in Alarm::iter() {
            std::env::set_var(alarm.to_string(), "3");
        }

        let config = super::get().expect("unable to get config from env vars");
        assert_eq!(config.len(), Alarm::iter().count());
        for (_alarm, config) in config {
            assert_eq!(config.value(), 3.);
        }

        for alarm in Alarm::iter() {
            std::env::remove_var(alarm.to_string());
        }

        // Parse Error
        std::env::set_var("CPU_LOWER_LIMIT_VCPUS", "a");
        assert!(super::get().is_err());

        // Zero
        std::env::set_var("CPU_LOWER_LIMIT_VCPUS", "0");
        let config = super::get().expect("unable to get config from env vars");
        assert!(config.is_empty());

        // Default
        std::env::set_var("CPU_LOWER_LIMIT_VCPUS", "5.");
        let config = super::get().expect("unable to get config from env vars");
        assert_eq!(config.len(), 1);

        let cpu_lower = config
            .get(&Alarm::CpuLowerLimitVcpus)
            .expect("no lower limit for cpu found");
        assert_eq!(cpu_lower.value(), 5.);
        assert_eq!(cpu_lower.period_minutes(), 1);
        assert_eq!(cpu_lower.data_points(), 5);
        assert_eq!(cpu_lower.data_points_to_alarm(), 3);

        // Clipped
        std::env::set_var("CPU_LOWER_LIMIT_VCPUS", "1");
        std::env::set_var("CPU_LOWER_LIMIT_VCPUS_PERIOD_MINUTES", "0");

        let config = super::get().expect("unable to get config from env vars");
        assert_eq!(config.len(), 1);

        let cpu_lower = config
            .get(&Alarm::CpuLowerLimitVcpus)
            .expect("no lower limit for cpu found");
        assert_eq!(cpu_lower.value(), 1.);
        assert_eq!(cpu_lower.period_minutes(), 1);

        // Custom
        // Setting env vars affects the whole process, so we avoid doing that from many tests
        std::env::set_var("PERIOD_MINUTES", "3");
        std::env::set_var("DATA_POINTS", "2");
        std::env::set_var("DATA_POINTS_TO_ALARM", "2");

        std::env::set_var("CPU_LOWER_LIMIT_VCPUS", "1");
        std::env::set_var("CPU_LOWER_LIMIT_VCPUS_PERIOD_MINUTES", "5");
        std::env::set_var("CPU_LOWER_LIMIT_VCPUS_DATA_POINTS", "6");
        std::env::set_var("CPU_LOWER_LIMIT_VCPUS_DATA_POINTS_TO_ALARM", "1");

        std::env::set_var("CPU_UPPER_LIMIT_VCPUS", "4");
        let config = super::get().expect("unable to get config from env vars");
        assert_eq!(config.len(), 2);

        let cpu_lower = config
            .get(&Alarm::CpuLowerLimitVcpus)
            .expect("no lower limit for cpu found");
        assert_eq!(cpu_lower.value(), 1.);
        assert_eq!(cpu_lower.period_minutes(), 5);
        assert_eq!(cpu_lower.data_points(), 6);
        assert_eq!(cpu_lower.data_points_to_alarm(), 1);

        let cpu_upper = config
            .get(&Alarm::CpuUpperLimitVcpus)
            .expect("no upper limit for cpu found");
        assert_eq!(cpu_upper.value(), 4.);
        assert_eq!(cpu_upper.period_minutes(), 3);
        assert_eq!(cpu_upper.data_points(), 2);
        assert_eq!(cpu_upper.data_points_to_alarm(), 2);
    }
}
