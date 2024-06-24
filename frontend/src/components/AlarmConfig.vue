<template>
  <div class="flex flex-col w-full">
    <div>
      <label for="api-token" class="mr-2">Railway API Token</label>
      <input name="api-token" type="password" v-model="apiToken" /> 
    </div>
    <div class="mt-1" v-if="apiToken?.trim().length">
      <label for="project-id" class="mr-2">Project</label>
      <select name="project-id" :value="projectId" @change="(event) => projectId = event.target.value"> 
        <option v-for="project in projects" :value="project.id" :key="project.id" :selected="project.id === projectId">{{project.name}}</option>
      </select>
    </div>
    <div class="mt-1" v-if="projectId?.trim()">
      <label for="service-id" class="mr-2">Monitored Service</label>
      <select name="service-id" :value="serviceId" @change="(event) => serviceId = event.target.value"> 
        <option v-for="service in monitoredServices" :value="service.id" :key="service.id" :selected="service.id === serviceId">{{service.name}}</option>
      </select>
    </div>
    <div class="mt-3" v-if="serviceId?.trim().length">
      <label for="health-check" class="mr-2">Health Check URL</label>
      <input name="health-check" type="text" v-model="healthCheck" /> 
    </div>
    <div class="mt-3">
      <label for="period-minutes" class="mr-2">Period (minutes)</label>
      <input name="period-minutes" type="number" v-model="periodMinutes" /> 
    </div>
    <div>
      <label for="data-points" class="mr-2">Data Points</label>
      <input name="data-points" type="number" v-model="dataPoints" /> 
    </div>
    <div>
      <label for="data-points-to-alarm" class="mr-2">Data Points to Alarm</label>
      <input name="data-points-to-alarm" type="number" v-model="dataPointsToAlarm" /> 
    </div>
    <a v-if="deployUrl" :href="deployUrl" class="flex self-center mt-2">
      <img src="https://railway.app/button.svg" title="Create Service From Template" />
    </a>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from "vue";
import _ from "lodash";

const apiToken = ref("");
const serviceId = ref("");
const projectId = ref("");

const periodMinutes = ref(1);
const dataPoints = ref(5);
const dataPointsToAlarm = ref(3);

const healthCheck = ref("");

const projects = ref<{ id: string, name: string }[]>([]);
const monitoredServices = ref<{ id: string, name: string }[]>([]);

const url = import.meta.env.BACKEND_URL ?? "http://localhost:4000";
console.log(url);

watch(apiToken, _.debounce(async () => {
  projects.value = [];

  if (!apiToken.value?.trim()) return;

  projects.value = await (await fetch(`${url}/v1/projects`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${apiToken.value}`,
    },
  })).json();
}, 500), { immediate: true })

watch([projectId, apiToken], async () => {
  monitoredServices.value = [];

  if (!projectId.value?.trim() || !apiToken.value) return;

  monitoredServices.value = await (await fetch(`${url}/v1/services`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${apiToken.value}`,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      "projectId": projectId.value,
    })
  })).json();
}, { immediate: true })

watch([serviceId, monitoredServices], () => {
  healthCheck.value = monitoredServices.value.find((s) => s.id === serviceId.value)?.healthCheckUrl ?? "";
}, { immediate: true })

const kv = (k: string, v: string) => `${k}=${v}`;

const deployUrl = computed(() => {
  if (!apiToken.value?.trim() || !projectId.value?.trim()?.length || !serviceId.value?.trim()?.length) return;

  const base = "https://railway.app/new/template/IS74og?referralCode=F6ei8i";
  const params = {
    RAILWAY_API_TOKEN: apiToken.value,
    RAILWAY_PROJECT_ID: projectId.value,
    RAILWAY_MONITORED_SERVICE_ID: serviceId.value,
    ALARM_TOKEN: `${Math.random().toString(36).slice(2)}${Math.random().toString(36).slice(2)}${Math.random().toString(36).slice(2)}`
  };
  return `${base}&${Object.entries(params).map(([k, v]) => kv(k, v)).join('&')}`;
})

</script>

<style scoped>
label {
  display: inline-block;
  width: 200px;
}
input, select {
  color: black;
  border-radius: 0.25rem;
  padding: 0.3rem;
}
</style>
