# Alarms V0

*This is a draft, it's still in progress*

Calls WebHook and sends PagerDuty events if the state changes for some alarm

Can monitor a HealthCheck endpoint and CPU, Memory, Disk, Ingress or Egress going over or under a threshold.

## Environment Variables:

### Required for basic usage:

- RAILWAY_API_TOKEN

  The GraphQL API token generated for you on Railway: https://docs.railway.app/guides/public-api#creating-a-token

- ALARM_TOKEN

  Key used to sign the WebHook with HMAC SHA256, signature is sent in the X-HUB-SIGNATURE-256 header

- RAILWAY_MONITORED_SERVICE_ID

  ID of service to monitor

- WebHook or PagerDuty

  To trigger any action from the alarm you must set the WebHook and/or PagerDuty integration, check their sections to properly configure them

### Limits

These are the alarms that can be configured.

- `CPU_LOWER_LIMIT_VCPUS`, `CPU_UPPER_LIMIT_VCPUS`
- `DISK_LOWER_LIMIT_GB`, `DISK_UPPER_LIMIT_GB`
- `EGRESS_LOWER_LIMIT_GB`, `EGRESS_UPPER_LIMIT_GB`
- `INGRESS_LOWER_LIMIT_GB`, `INGRESS_UPPER_LIMIT_GB`
- `MEMORY_LOWER_LIMIT_GB`, `MEMORY_UPPER_LIMIT_GB`,

The alarm thresholds can be set by defining one or many of those environment variables above. If the variable is unset or 0 it's measurement will be ignored.

The alarm will be emitted if at least one of those limits is breached or stops breaching. The WebHook will receive the alarms that changed + all alarms that are active at the moment to enable the combination of them.

### Interval configuration

It's possible to configure the details that will control the alarm, like the interval between measurements, the number of data-points to analyze and the minimal number of breaching data-points that will trigger an alarm.

To set a global configuration use the following environment variables.

- `PERIOD_MINUTES` is the number of minutes to use to evaluate the metric to create each individual data point for an alarm.

- `DATA_POINTS` is the number of the most recent periods, or data points, to evaluate when determining alarm state.

- `DATA_POINTS_TO_ALARM` is the number of data points within the last `DATA_POINTS` that must be breaching to cause the alarm to go to the ALARM state. The breaching data points don't have to be consecutive, but they must all be within the last number of data points equal to `DATA_POINTS`

It's also possible to granularly configure the alarm by prefixing the environment variables above with the threshold related to the configuration, like `MEMORY_LOWER_LIMIT_GB_PERIOD_MINUTES` to configure the threshold `MEMORY_LOWER_LIMIT_GB`. `DATA_POINTS` and `DATA_POINTS_TO_ALARM` can also be configured in similar ways

If no specific configuration exists for one alarm threshold it will use the global ones, if no global configuration is set: `PERIOD_MINUTES` will be `1`, `DATA_POINTS` will be `5` and `DATA_POINTS_TO_ALARM` will be `3`.

### Set by Railway:

- RAILWAY_PROJECT_ID

  Project ID for the resources being monitored. It's set by default by Railway, no need to change unless monitoring a service from another project

## Example configuration

```
CPU_UPPER_LIMIT_VCPUS=10
CPU_UPPER_LIMIT_VCPUS_PERIOD_MINUTES=5 # Will get average usage during those minutes
CPU_UPPER_LIMIT_VCPUS_DATA_POINTS=5 # 25 minutes analyzed in total
CPU_UPPER_LIMIT_VCPUS_DATA_POINTS_TO_ALARM=3 # If there is a breach for 15 minutes of the 25 (even if non consecutive) triggers alarm

HEALTH_CHECK_FAILED=https://my-endpoint.com/healthcheck

# Will use default period/data-points/data-points to alarm
CPU_LOWER_LIMIT_VCPUS=0.1

RAM_UPPER_LIMIT_GB=10
RAM_LOWER_LIMIT_GB=0.1
```

### Healthcheck

To set the healthcheck GET endpoint use the `HEALTH_CHECK_FAILED` environment variable. The granular configuration above is also available for it.

The healthcheck endpoint must return a 200 status code to be computed as working.

**Tip: use the same healthcheck endpoint you use for Railway's initial healthcheck test for the service.**

Example:

```
HEALTH_CHECK_FAILED=https://my-url.com/healthcheck
```

## PagerDuty Alarms

To configure the PagerDuty integration you must specify the following environment variables
  - `PAGER_DUTY_TOKEN`: specify the authentication token for PagerDuty's API
  - `PAGER_DUTY_SOURCE`: the `RAILWAY_PUBLIC_DOMAIN` for the monitored service
  - `PAGER_DUTY_ROUTING_KEY`: The GUID of one of your PagerDuty Events API V2 integrations.
     This is the "Integration Key" listed on the Events API V2 integration's detail page.

The environment variable `PAGER_DUTY_URL` can also be set to override the default's PagerDuty endpoint.

An alert event will be created for each alarm state change.

## WebHook API

The endpoint specified by the environment variable `WEB_HOOK_URL` will be called if at least one alarm changed state. All active alarms will also be sent in that WebHook request, even if their state wasn't the one that changed.

The JSON payload is signed with HMAC SHA256 and sent in the `X-HUG-SIGNATURE-256` HTTP header. The schema of the payload is described below:

```
interface Payload {
    serviceId: string;
    alarms: {
        on: boolean;
        alarm: 'CPU_LOWER_LIMIT_VCPUS'
               | 'CPU_UPPER_LIMIT_VCPUS'
               | 'DISK_LOWER_LIMIT_GB'
               | 'DISK_UPPER_LIMIT_GB'
               | 'EGRESS_LOWER_LIMIT_GB'
               | 'EGRESS_UPPER_LIMIT_GB'
               | 'INGRESS_LOWER_LIMIT_GB'
               | 'INGRESS_UPPER_LIMIT_GB'
               | 'MEMORY_LOWER_LIMIT_GB'
               | 'MEMORY_UPPER_LIMIT_GB';
    }[];
}
```

The WebHook won't be retried, even if a non 200 response is received.

## Future Work

TODO

V0:
- Discord integration
- Retry WebHook if a non 200 response is received
- Add warm-up period leniency for new deployments for healthcheck
- Healthcheck each replica
- Allow adding an action to alarm: reboot/redeploy/stop
    - Reboot unhealthy service instances for example
    - Reboot on memory leak (if RAM > 80% for 2 hours)
    - Add one more replica if CPU usage is too large
    - etc
- Configure alarm when >=, >, <= or <
- Add INSUFFICIENT_DATA state
- Persist alarms, display graphs with them over time
  - Alarm change is lost if it changed while the alarm service was down, will require manual intervention in pager-duty/WebHook

V0.5
- slack + email integration
- cost alarms
- Alarm when deployment crashes
- Endpoint to get current alarm state
- Integrate horizontal auto-scale with it

V1
- Have more ergonomic interface than environment variables
- Make GraphQL subscription for alarms
- Toast & Notification in front-end
- Integrate with service instance size limits (RAM and CPU), allowing for percentage thresholds
- Don't fetch usage every minute if not needed, use GraphQL API's start and end date more optimally
- Add description to alarm/service

V2
- Period in seconds, not minutes
- Add percentile based metrics
- Math expression alarms combining multiple metrics
    - They all must have same period
- Alarm based on data sources
- Alarm on logs

