# Alarms

*This is a draft, it's still in progress*

Calls WebHook if the state change for some alarm defined through environment variables

Can monitor CPU, Memory, Disk, Ingress or Egress going over or under a threshold.

Eventually should emit a pager-duty event, send a discord message, a slack message and an email.

It should also monitor the health-check endpoint and costs, emitting alarms for them.

## Environment Variables:

### Required for basic usage:

- RAILWAY_API_TOKEN

  The GraphQL API token generated for you on Railway: https://docs.railway.app/guides/public-api#creating-a-token

- ALARM_TOKEN

  Key used to sign the WebHook with HMAC SHA256, signature is sent in the X-HUB-SIGNATURE-256 header

- RAILWAY_SERVICE_ID

  ID of service to monitor

- WEBHOOK_URL

  URL of WebHook called when at least one alarm's state changes, won't be required once other channels are integrated, but it's needed for now

### Thresholds

These are the alarms that can be configured.

- `CPU_LOWER_LIMIT_VCPUS`, `CPU_UPPER_LIMIT_VCPUS`
- `DISK_LOWER_LIMIT_GB`, `DISK_UPPER_LIMIT_GB`
- `EGRESS_LOWER_LIMIT_GB`, `EGRESS_UPPER_LIMIT_GB`
- `INGRESS_LOWER_LIMIT_GB`, `INGRESS_UPPER_LIMIT_GB`
- `MEMORY_LOWER_LIMIT_GB`, `MEMORY_UPPER_LIMIT_GB`,

The alarm threshold can be set by defining one or many of those limits, by setting the environment variable with the same name. If the variable is unset or 0 it will be ignored.

The alarm will be emitted if at least one of those limits is breached, if more than one alarm is on at the same time, both will be sent in the WebHook payload

### Interval configuration

It's possible to configure the details that will control the alarm, like the interval between measurements, the number of data-points to analyze and the minimal number of breaching data-points that will trigger an alarm.

To set a generic configuration use the following environment variables.

- `PERIOD_MINUTES` is the number of minutes to use to evaluate the metric to create each individual data point for an alarm.

- `DATA_POINTS` is the number of the most recent periods, or data points, to evaluate when determining alarm state.

- `DATA_POINTS_TO_ALARM` is the number of data points within the last `DATA_POINTS` that must be breaching to cause the alarm to go to the ALARM state. The breaching data points don't have to be consecutive, but they must all be within the last number of data points equal to `DATA_POINTS`

It's also possible to granularly configure the alarm by prefixing the environment variables above with the threshold related to the configuration, like `MEMORY_LOWER_LIMIT_GB_PERIOD_MINUTES` to configure the threshold `MEMORY_LOWER_LIMIT_GB`. `DATA_POINTS` and `DATA_POINTS_TO_ALARM` can also be configured in similar ways

If no specific configuration exists for one alarm threshold, it will use the generic ones, if no generic configuration is set the alarm `PERIOD_MINUTES` will be `1`, `DATA_POINTS` will be `5` and `DATA_POINTS_TO_ALARM` will be `3`.

### Set by Railway:

- RAILWAY_PROJECT_ID

  Project ID for the resources being monitored. It's set by default by Railway, no need to change unless monitoring a service from another project

## Example configuration

```
CPU_UPPER_LIMIT_VCPUS=10
CPU_UPPER_LIMIT_VCPUS_PERIOD_MINUTES=5 # Will get average usage during those minutes
CPU_UPPER_LIMIT_VCPUS_DATA_POINTS=5 # 25 minutes analyzed in total
CPU_UPPER_LIMIT_VCPUS_DATA_POINTS_TO_ALARM=3 # If there is a breach for 15 minutes of the 25 (even if non consecutive) triggers alarm

# Will use default period/data-points/data-points to alarm
CPU_LOWER_LIMIT_VCPUS=0.1 

RAM_UPPER_LIMIT_GB=10
RAM_LOWER_LIMIT_GB=0.1
```

## WebHook API

The endpoint specified by the environment variable `WEBHOOK_URL` will be called if at least one alarm changed state. All active alarms will be sent in that WebHook, even if their state wasn't the one that changed.

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

- Retry WebHook if a non 200 response is received
- Integrate with service instance size limits (RAM and CPU), allowing for percentage thresholds
- Have more ergonomic interface than environment variables
- Endpoint to get current alarm state
- Period in seconds, not minutes
- Don't fetch usage every minute if not needed, use GraphQL API's start and end date more optimally
- Healthcheck failure and cost alarms
- Pager-Duty + Discord + Slack + Email
- Make GraphQL subscription for it
- Toast & Notification in front-end
