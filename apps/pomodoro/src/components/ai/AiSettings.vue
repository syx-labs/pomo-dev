<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { getAllSettings, setSetting, invokeAiCommand } from "@/lib/tauri";
import { useToast } from "@/composables/useToast";
import OllamaModels from "./OllamaModels.vue";

const { showToast } = useToast();

type Provider = "disabled" | "ollama" | "openai" | "anthropic";

const MODEL_DEFAULTS: Record<Exclude<Provider, "disabled">, string> = {
  ollama: "llama3.2",
  openai: "gpt-4o-mini",
  anthropic: "claude-sonnet-4-20250514",
};

const provider = ref<Provider>("disabled");
const apiKey = ref("");
const model = ref("");
const baseUrl = ref("http://localhost:11434");
const sendProjectNames = ref(false);
const testing = ref(false);

const showApiKey = ref<Exclude<Provider, "disabled" | "ollama">[]>(["openai", "anthropic"]);
const showBaseUrl = ref<Provider[]>(["ollama"]);

onMounted(async () => {
  try {
    const raw = await getAllSettings();
    provider.value = (raw["ai_provider"] as Provider) || "disabled";
    apiKey.value = raw["ai_api_key"] || "";
    model.value = raw["ai_model"] || "";
    baseUrl.value = raw["ai_base_url"] || "http://localhost:11434";
    sendProjectNames.value = raw["ai_send_project_names"] === "true";

    // Set default model if empty
    if (!model.value && provider.value !== "disabled") {
      model.value = MODEL_DEFAULTS[provider.value];
    }
  } catch {
    // Settings load failed silently
  }
});

watch(provider, (val, oldVal) => {
  if (val !== "disabled" && val !== oldVal) {
    model.value = MODEL_DEFAULTS[val];
  }
  save("ai_provider", val);
});

function save(key: string, value: string) {
  setSetting(key, value).catch(() => {
    showToast("Failed to save setting", "error");
  });
}

function onApiKeyChange(e: Event) {
  apiKey.value = (e.target as HTMLInputElement).value;
  save("ai_api_key", apiKey.value);
}

function onModelChange(e: Event) {
  model.value = (e.target as HTMLInputElement).value;
  save("ai_model", model.value);
}

function onBaseUrlChange(e: Event) {
  baseUrl.value = (e.target as HTMLInputElement).value;
  save("ai_base_url", baseUrl.value);
}

function toggleProjectNames() {
  sendProjectNames.value = !sendProjectNames.value;
  save("ai_send_project_names", String(sendProjectNames.value));
}

async function testConnection() {
  testing.value = true;
  try {
    const result = await invokeAiCommand("Hello, respond with OK");
    if (result.success) {
      showToast("Connection successful!", "success");
    } else {
      showToast(result.message || "Connection failed", "error");
    }
  } catch {
    showToast("Connection test failed", "error");
  } finally {
    testing.value = false;
  }
}
</script>

<template>
  <div class="ai-settings">
    <div class="field">
      <label class="field-label">AI Provider</label>
      <select v-model="provider" class="field-input select-input">
        <option value="disabled">Disabled</option>
        <option value="ollama">Ollama</option>
        <option value="openai">OpenAI</option>
        <option value="anthropic">Anthropic</option>
      </select>
    </div>

    <template v-if="provider !== 'disabled'">
      <div v-if="(showApiKey as string[]).includes(provider)" class="field">
        <label class="field-label">API Key</label>
        <input
          type="password"
          class="field-input"
          :value="apiKey"
          placeholder="sk-..."
          @change="onApiKeyChange"
        />
      </div>

      <div class="field">
        <label class="field-label">Model</label>
        <input
          type="text"
          class="field-input"
          :value="model"
          :placeholder="MODEL_DEFAULTS[provider as Exclude<Provider, 'disabled'>]"
          @change="onModelChange"
        />
      </div>

      <div v-if="(showBaseUrl as string[]).includes(provider)" class="field">
        <label class="field-label">Base URL</label>
        <input
          type="text"
          class="field-input"
          :value="baseUrl"
          placeholder="http://localhost:11434"
          @change="onBaseUrlChange"
        />
      </div>

      <div class="field row">
        <label class="field-label">
          <span>Include project names in AI prompts</span>
        </label>
        <button class="toggle" :class="{ active: sendProjectNames }" @click="toggleProjectNames">
          <span class="toggle-knob" />
        </button>
      </div>

      <button class="test-btn" :disabled="testing" @click="testConnection">
        <template v-if="testing">
          <svg class="spinner" width="14" height="14" viewBox="0 0 14 14">
            <circle
              cx="7"
              cy="7"
              r="5.5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-dasharray="20 14"
            />
          </svg>
          Testing...
        </template>
        <template v-else>Test Connection</template>
      </button>

      <OllamaModels v-if="provider === 'ollama'" />
    </template>
  </div>
</template>

<style scoped>
.ai-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field.row {
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
}

.field-label {
  font-size: 14px;
  color: var(--text-secondary);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.field-input {
  background: var(--bg-card);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 14px;
  padding: 8px 12px;
  outline: none;
  transition: border-color 0.15s;
  width: 100%;
  box-sizing: border-box;
}

.field-input:focus {
  border-color: var(--color-work);
}

.select-input {
  appearance: none;
  -webkit-appearance: none;
  cursor: pointer;
  background-image: url("data:image/svg+xml,%3Csvg width='12' height='12' viewBox='0 0 12 12' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M3 4.5L6 7.5L9 4.5' stroke='%2371717a' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  padding-right: 30px;
}

.toggle {
  position: relative;
  width: 44px;
  height: 24px;
  border-radius: 12px;
  background: var(--bg-hover);
  border: none;
  cursor: pointer;
  padding: 0;
  flex-shrink: 0;
  transition: background 0.2s;
}

.toggle.active {
  background: var(--color-work);
}

.toggle-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: var(--toggle-knob-bg);
  transition: transform 0.2s;
}

.toggle.active .toggle-knob {
  transform: translateX(20px);
}

.test-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  background: none;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  cursor: pointer;
  align-self: flex-start;
  transition:
    color 0.15s,
    border-color 0.15s;
}

.test-btn:hover:not(:disabled) {
  color: var(--text-primary);
  border-color: var(--text-muted);
}

.test-btn:active:not(:disabled) {
  transform: scale(0.97);
}

.test-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
