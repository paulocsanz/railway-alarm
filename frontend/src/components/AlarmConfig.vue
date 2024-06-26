<template>
  <div class="flex flex-col w-full mt-5 w-full">
    <div class="flex">
      <label for="api-token" class="mr-2">Railway API Token</label>
      <input name="api-token" type="password" v-model="apiToken" class="grow" />
    </div>
    <div class="mt-1 flex" v-if="projects.length">
      <label for="project-id" class="mr-2 mt-1">Project</label>
      <select name="project-id" v-model="projectId" class="grow">
        <option
          v-for="project in projects"
          :value="project.id"
          :key="project.id"
          :selected="project.id === projectId"
        >
          {{ project.name }}
        </option>
      </select>
    </div>

    <div class="mt-1 flex" v-if="projectId?.trim()">
      <label for="service-id" class="mr-2 mt-1">Monitored Service</label>
      <select name="service-id" v-model="serviceId" class="grow">
        <option
          v-for="service in monitoredServices"
          :value="service.id"
          :key="service.id"
          :selected="service.id === serviceId"
        >
          {{ service.name }}
        </option>
      </select>
    </div>

    <template v-if="serviceId?.trim().length">
      <div class="mt-3 flex">
        <label for="webhook" class="mr-2 mt-1">WebHook URL</label>
        <input name="webhook" type="text" v-model="webhook" class="grow" />
      </div>

      <div class="mt-3 flex flex-col" v-if="service">
        <span class="text-xl">Pager Duty</span>

        <div class="mt-1">
          <div class="flex">
            <label for="pagerduty-token" class="mr-2">API Token</label>
            <input name="pagerduty-token" type="text" v-model="pagerdutyToken" class="grow" />
          </div>
          <p class="text-xs">Events V2 API Key</p>
          <p class="text-xs">https://&lt;customdomain&gt;.pagerduty.com/api_keys</p>
          <p class="text-xs">
            <a href="https://support.pagerduty.com/docs/api-access-keys"
              >https://support.pagerduty.com/docs/api-access-keys</a
            >
          </p>
        </div>

        <div class="mt-1">
          <div class="flex">
            <label for="pagerduty-routing-key" class="mr-2">Integration/Routing Key</label>
            <input
              name="pagerduty-routing-key"
              type="text"
              v-model="pagerdutyRoutingKey"
              class="grow"
            />
          </div>
          <div class="text-xs">Integration Key for Events v2 API</div>
          <div class="text-xs inline-block">
            Create PagerDuty integration and copy integration key:
            <a
              href="https://support.pagerduty.com/docs/services-and-integrations#section-events-API-v2"
              >https://support.pagerduty.com/docs/services-and-integrations#section-events-API-v2</a
            >
          </div>
          <div class="text-xs">
            <a
              href="https://developer.pagerduty.com/docs/ZG9jOjExMDI5NTgw-events-api-v2-overview#getting-started"
            >
              https://developer.pagerduty.com/docs/ZG9jOjExMDI5NTgw-events-api-v2-overview#getting-started
            </a>
          </div>
        </div>
      </div>
    </template>

    <template v-if="serviceId?.trim().length">
      <div class="mt-10">
        <label for="period-minutes" class="mr-2">Period (minutes)</label>
        <input name="period-minutes" type="number" v-model="periodMinutes" />
      </div>
      <div class="mt-1">
        <label for="data-points" class="mr-2">Data Points</label>
        <input name="data-points" type="number" v-model="dataPoints" />
      </div>
      <div class="mt-1">
        <label for="data-points-to-alarm" class="mr-2">Data Points to Alarm</label>
        <input name="data-points-to-alarm" type="number" v-model="dataPointsToAlarm" />
      </div>

      <div class="mt-3">
        <label for="health-check" class="mr-2 mt-1">Health Check URL</label>
        <input name="health-check" type="text" v-model="healthCheck" />
      </div>

      <span v-if="alarms.length" class="mt-3 font-bold text-2xl">Alarm When:</span>
      <div class="flex" v-for="alarm in alarms" :key="`${alarm.metric}/${alarm.operator}`">
        <span class="ml-2 mt-0.5 text-xl"
          >{{ alarm.metric }} {{ alarm.operator }} {{ alarm.value }}</span
        >
        <svg
          @click="
            alarms = alarms.filter(
              (a) => !(a.metric === alarm.metric && a.operator === alarm.operator)
            )
          "
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="currentColor"
          class="size-6 ml-2 mt-1 text-red-400 cursor-pointer"
        >
          <path
            fill-rule="evenodd"
            d="M16.5 4.478v.227a48.816 48.816 0 0 1 3.878.512.75.75 0 1 1-.256 1.478l-.209-.035-1.005 13.07a3 3 0 0 1-2.991 2.77H8.084a3 3 0 0 1-2.991-2.77L4.087 6.66l-.209.035a.75.75 0 0 1-.256-1.478A48.567 48.567 0 0 1 7.5 4.705v-.227c0-1.564 1.213-2.9 2.816-2.951a52.662 52.662 0 0 1 3.369 0c1.603.051 2.815 1.387 2.815 2.951Zm-6.136-1.452a51.196 51.196 0 0 1 3.273 0C14.39 3.05 15 3.684 15 4.478v.113a49.488 49.488 0 0 0-6 0v-.113c0-.794.609-1.428 1.364-1.452Zm-.355 5.945a.75.75 0 1 0-1.5.058l.347 9a.75.75 0 1 0 1.499-.058l-.346-9Zm5.48.058a.75.75 0 1 0-1.498-.058l-.347 9a.75.75 0 0 0 1.5.058l.345-9Z"
            clip-rule="evenodd"
          />
        </svg>
      </div>
    </template>

    <a v-if="deployUrl && alarms.length" :href="deployUrl" class="self-center" target="_blank">
      <img class="mt-5" src="https://railway.app/button.svg" title="Create Service From Template" />
    </a>

    <template v-if="serviceId?.trim().length">
      <div class="mt-10 flex flex-col">
        <p class="text-2xl">Add new alarm</p>

        <p class="text-red-400" v-if="errorMessage">{{ errorMessage }}</p>

        <div>
          <label for="type" class="mr-2 mt-1">Metric:</label>
          <select name="metric" v-model="metric">
            <option v-for="m in metrics" :value="m" :key="m" :selected="metric === m">
              {{ m }}
            </option>
          </select>
        </div>

        <div class="mt-1">
          <label for="operator" class="mr-2 mt-1">Operator:</label>
          <select name="operator" v-model="operator">
            <option value=">">&gt;</option>
            <!--<option value=">=">&ge;</option>-->
            <option value="<">&lt;</option>
            <!--<option value="<=">&le;</option>-->
          </select>
        </div>

        <div class="mt-1" v-if="metric">
          <label for="value" class="mr-2 mt-1">Limit ({{ unit(metric) }}):</label>
          <input name="value" type="number" v-model="value" />
        </div>

        <button @click="addAlarm" class="my-2 p-2 border">Add alarm</button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import _ from 'lodash'

const apiToken = ref(localStorage.getItem('token') ?? '')
const serviceId = ref('')
const projectId = ref('')
const webhook = ref('')

const periodMinutes = ref(1)
const dataPoints = ref(5)
const dataPointsToAlarm = ref(3)

const healthCheck = ref('')

const projects = ref<{ id: string; name: string }[]>([])
const monitoredServices = ref<{ id: string; name: string; healthCheckUrl?: string }[]>([])
const alarms = ref<{ metric: Metric; operator: '>' | '<'; value: number }[]>([])

const service = computed(() => {
  return monitoredServices.value.find((s) => s.id === serviceId.value)
})

const pagerdutyToken = ref('')
const pagerdutyRoutingKey = ref('')
const pagerdutySource = computed(() =>
  service.value ? `\$\{${service.value.name}.RAILWAY_PUBLIC_DOMAIN\}` : null
)

type Metric = 'CPU' | 'RAM' | 'Disk' | 'Ingress' | 'Egress'
const metrics: Metric[] = ['CPU', 'RAM', 'Disk', 'Ingress', 'Egress']

const metric = ref<Metric>('CPU')
const operator = ref<'>' | '<'>('>')
const value = ref(0)

const url = import.meta.env.PROD
  ? 'https://backend-production-9dbc.up.railway.app'
  : 'http://localhost:4000'

const unit = (metric: Metric) => {
  switch (metric) {
    case 'CPU':
      return 'vCPU'
    case 'RAM':
      return 'GB'
    case 'Disk':
      return 'GB'
    case 'Ingress':
      return 'GB'
    case 'Egress':
      return 'GB'
    default:
      return ''
  }
}

const errorMessage = ref('')
const addAlarm = () => {
  errorMessage.value = ''
  if (
    alarms.value.filter((a) => a.metric === metric.value).find((a) => a.operator === operator.value)
  ) {
    errorMessage.value = 'Alarm already defined'
    return
  }

  if (value.value === 0) {
    errorMessage.value = "Threshold can't be zero"
    return
  }

  alarms.value.push({
    metric: metric.value,
    operator: operator.value,
    value: value.value
  })
}

watch(
  apiToken,
  _.debounce(async () => {
    projects.value = []

    if (!apiToken.value?.trim()) return

    localStorage.setItem('token', apiToken.value.trim())

    projects.value = await (
      await fetch(`${url}/v1/projects`, {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${apiToken.value}`
        }
      })
    ).json()
  }, 500),
  { immediate: true }
)

watch(
  [projectId, apiToken],
  async () => {
    monitoredServices.value = []

    if (!projectId.value?.trim() || !apiToken.value) return

    monitoredServices.value = await (
      await fetch(`${url}/v1/services`, {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${apiToken.value}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          projectId: projectId.value
        })
      })
    ).json()
  },
  { immediate: true }
)

watch(
  [serviceId, monitoredServices],
  () => {
    healthCheck.value =
      monitoredServices.value.find((s) => s.id === serviceId.value)?.healthCheckUrl ?? ''
  },
  { immediate: true }
)

const kv = (k: string, v: string) => `${k}=${v}`

const deployUrl = computed(() => {
  if (
    !apiToken.value?.trim() ||
    !projectId.value?.trim()?.length ||
    !serviceId.value?.trim()?.length
  )
    return

  const base = 'https://railway.app/new/template/IS74og?referralCode=F6ei8i'
  const params: Record<string, string | number> = {
    RAILWAY_API_TOKEN: apiToken.value,
    RAILWAY_PROJECT_ID: projectId.value,
    RAILWAY_MONITORED_SERVICE_ID: serviceId.value,
    ALARM_TOKEN: `${Math.random().toString(36).slice(2)}${Math.random().toString(36).slice(2)}${Math.random().toString(36).slice(2)}`
  }

  if (healthCheck.value.trim()) {
    params['HEALTH_CHECK_FAILED'] = healthCheck.value
  }

  if (periodMinutes.value) {
    params['PERIOD_MINUTES'] = periodMinutes.value
  }

  if (dataPoints.value) {
    params['DATA_POINTS'] = dataPoints.value
  }

  if (dataPointsToAlarm.value) {
    params['DATA_POINTS_TO_ALARM'] = dataPointsToAlarm.value
  }

  if (webhook.value.trim().length) {
    params['WEB_HOOK_URL'] = webhook.value.trim()
  }

  if (
    pagerdutyToken.value.trim().length &&
    pagerdutyRoutingKey.value.trim() &&
    pagerdutySource.value
  ) {
    params['PAGER_DUTY_TOKEN'] = pagerdutyToken.value.trim()
    params['PAGER_DUTY_ROUTING_KEY'] = pagerdutyRoutingKey.value.trim()
    params['PAGER_DUTY_SOURCE'] = pagerdutySource.value
  }

  const envUnit = (metric: Metric) => {
    switch (metric) {
      case 'CPU':
        return 'VCPUS'
      case 'RAM':
        return 'GB'
      case 'Disk':
        return 'GB'
      case 'Ingress':
        return 'GB'
      case 'Egress':
        return 'GB'
      default:
        throw new Error('Unknown metric')
    }
  }

  const envMetric = (metric: Metric) => {
    switch (metric) {
      case 'CPU':
        return 'CPU'
      case 'RAM':
        return 'MEMORY'
      case 'Disk':
        return 'DISK'
      case 'Ingress':
        return 'INGRESS'
      case 'Egress':
        return 'EGRESS'
      default:
        throw new Error('Unknown metric')
    }
  }

  for (const alarm of alarms.value) {
    const path = `${envMetric(alarm.metric)}_${alarm.operator === '<' ? 'LOWER' : 'UPPER'}_LIMIT_${envUnit(alarm.metric)}`
    params[path] = alarm.value
  }

  return `${base}&${Object.entries(params)
    .map(([k, v]) => kv(k, `${v}`))
    .join('&')}`
})
</script>

<style scoped>
label {
  display: inline-block;
  width: 200px;
}
input,
select {
  color: black;
  border-radius: 0.25rem;
  padding: 0.3rem;
}
</style>
