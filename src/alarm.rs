use crate::{AlarmState, Error, Result};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use tracing::error;

#[remain::sorted]
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Hash, Display, EnumIter)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Alarm {
    CostLowerLimit,
    CostUpperLimit,
    CpuLowerLimit,
    CpuUpperLimit,
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

pub async fn emit(alarms: Vec<AlarmState>, auth: &str, service_id: &str) {
    if alarms.is_empty() {
        return;
    }

    if let Err(err) = webhook(&alarms, auth, service_id).await {
        error!("Unable to send webhook for alarms: {err} - {alarms:#?}")
    }
}

async fn webhook(alarms: &[AlarmState], auth: &str, service_id: &str) -> Result<()> {
    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct WebHookPayload<'a> {
        alarms: &'a [AlarmState],
        service_id: &'a str,
    }

    if let Ok(url) = std::env::var("WEBHOOK_URL") {
        let payload = WebHookPayload { alarms, service_id };
        let body = serde_json::to_vec(&payload)?;
        let signature = hash(auth, &body)?;
        let response = reqwest::Client::new()
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-HUB-SIGNATURE-256", signature)
            .body(body)
            .fetch_mode_no_cors()
            .send()
            .await?;

        let status = response.status();
        if status != 200 {
            return Err(Error::WebHookStatusFailure(
                status.as_u16(),
                response.text().await?,
            ));
        }
    }
    Ok(())
}

fn hash(secret: &str, payload: &[u8]) -> Result<String> {
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes())?;
    mac.update(payload);
    let result = mac.finalize().into_bytes();
    Ok(format!("{result:x}"))
}
