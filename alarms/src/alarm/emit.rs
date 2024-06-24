use crate::{Alarm, AlarmPayload, AlarmState, Error, Result};
use hmac::{Hmac, Mac};
use serde::Serialize;
use std::collections::HashMap;
use tracing::{error, info, debug};

pub async fn emit(
    mut alarms: HashMap<Alarm, AlarmState>,
    alarm_payloads: &HashMap<Alarm, AlarmPayload>,
    auth: &str,
    service_id: &str,
) {
    if alarms.is_empty() {
        return;
    }

    debug!("Alarm ({service_id}): {alarms:?}");

    if let Err(err) = pager_duty(&alarms, service_id).await {
        error!("Unable to send pager duty events for alarms: {err} - {alarms:#?}")
    }

    // Populates webhook alarms with all active alarms to allow combining them arbitrarily on the other side
    for (alarm, payload) in &*alarm_payloads {
        if payload.state() {
            alarms.insert(*alarm, AlarmState::new(*alarm, payload.state()));
        }
    }

    if let Err(err) = webhook(&alarms, auth, service_id).await {
        error!("Unable to send webhook for alarms: {err} - {alarms:#?}")
    }
}

async fn pager_duty(alarms: &HashMap<Alarm, AlarmState>, service_id: &str) -> Result<()> {
    let url = std::env::var("PAGER_DUTY_URL")
        .unwrap_or_else(|_| "https://events.pagerduty.com".to_owned());
    let url = format!("{url}/v2/enqueue");

    if let (Ok(token), Ok(source), Ok(routing_key)) = (
        std::env::var("PAGER_DUTY_TOKEN"),
        std::env::var("PAGER_DUTY_SOURCE"),
        std::env::var("PAGER_DUTY_ROUTING_KEY"),
    ) {
        info!("Sending actions to pager-duty {url}");

        for state in alarms.values() {
            let event_action = if state.on() {
                "trigger"
            } else {
                "resolve"
            };
            let response = reqwest::Client::new()
                .post(&url)
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({
                    "routing_key": routing_key,
                    "payload": {
                        "source": source,
                        // TODO: allow customizing severity
                        "severity": "error",
                        // TODO: add more metadata about the breaching of the alarm
                        "summary": format!("Railway Alarm {} breached for {source}: {service_id}", state.alarm()),
                        "class": state.alarm().to_string(),
                    },
                    // TODO: add replica metadata
                    "dedup_key": format!("{service_id}-{}", state.alarm()),
                    "event_action": event_action,
                }))
                .fetch_mode_no_cors()
                .send()
                .await
                .map_err(|err| Error::WebHookFailure(err, url.clone()))?;

            let status = response.status();
            if status != 200 && status != 202 {
                return Err(Error::WebHookStatusFailure(
                    status.as_u16(),
                    response
                        .text()
                        .await
                        .map_err(|err| Error::WebHookBody(err, url))?,
                ));
            }
        }
    }

    Ok(())
}

async fn webhook(alarms: &HashMap<Alarm, AlarmState>, auth: &str, service_id: &str) -> Result<()> {
    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct WebHookPayload<'a> {
        alarms: Vec<&'a AlarmState>,
        service_id: &'a str,
    }

    if let Ok(url) = std::env::var("WEB_HOOK_URL") {
        info!("Sending actions to webhook {url}");

        let payload = WebHookPayload {
            alarms: alarms.iter().map(|(_, v)| v).collect(),
            service_id,
        };
        let body = serde_json::to_vec(&payload)?;
        let signature = hash(auth, &body)?;
        let response = reqwest::Client::new()
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-HUB-SIGNATURE-256", signature)
            .body(body)
            .fetch_mode_no_cors()
            .send()
            .await
            .map_err(|err| Error::WebHookFailure(err, url.clone()))?;

        let status = response.status();
        if status != 200 {
            return Err(Error::WebHookStatusFailure(
                status.as_u16(),
                response
                    .text()
                    .await
                    .map_err(|err| Error::WebHookBody(err, url))?,
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
