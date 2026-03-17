<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
import { useSettingsStore, type AppSettings } from "@/stores/settings";
import { useSoundStore } from "@/stores/sound";
import { useIntegrationsStore } from "@/stores/integrations";
import SoundMixer from "@/components/sound/SoundMixer.vue";
import SoundPresets from "@/components/sound/SoundPresets.vue";
import IntegrationCard from "@/components/integrations/IntegrationCard.vue";
import WebhookConfig from "@/components/integrations/WebhookConfig.vue";
import SlackConfig from "@/components/integrations/SlackConfig.vue";
import DiscordConfig from "@/components/integrations/DiscordConfig.vue";
import EventLog from "@/components/integrations/EventLog.vue";
import ConfirmDialog from "@/components/tasks/ConfirmDialog.vue";
import AiSettings from "@/components/ai/AiSettings.vue";
import type { IntegrationConfig, EventLogEntry } from "@/lib/tauri";

const store = useSettingsStore();
const soundStore = useSoundStore();
const intStore = useIntegrationsStore();

onMounted(() => {
  if (soundStore.availableSounds.length === 0) {
    soundStore.init();
  }
});

onMounted(() => {
  store.load();
  intStore.fetchIntegrations();
  intStore.fetchEventLog();
});

function onNumberChange(key: keyof AppSettings, event: Event) {
  const target = event.target as HTMLInputElement;
  store.update(key, Number(target.value));
}

function onToggle(key: keyof AppSettings) {
  const current = store.settings[key];
  if (typeof current === "boolean") {
    store.update(key, !current);
  }
}

function onRangeChange(key: keyof AppSettings, event: Event) {
  const target = event.target as HTMLInputElement;
  store.update(key, Number(target.value));
}

// Integrations state
const addDropdownOpen = ref(false);
const configType = ref<"webhook" | "slack" | "discord" | null>(null);
const configVisible = ref(false);
const editingIntegration = ref<IntegrationConfig | undefined>(undefined);
const deleteTarget = ref<IntegrationConfig | null>(null);
const confirmDeleteVisible = ref(false);

function openAddDropdown() {
  addDropdownOpen.value = !addDropdownOpen.value;
}

function startAdd(type: "webhook" | "slack" | "discord") {
  addDropdownOpen.value = false;
  editingIntegration.value = undefined;
  configType.value = type;
  configVisible.value = true;
}

function startEdit(integration: IntegrationConfig) {
  editingIntegration.value = integration;
  configType.value = integration.integration_type as "webhook" | "slack" | "discord";
  configVisible.value = true;
}

function closeConfig() {
  configVisible.value = false;
  editingIntegration.value = undefined;
  configType.value = null;
}

function handleWebhookSave(data: {
  name: string;
  url: string;
  headers: Record<string, string>;
  events: string[];
}) {
  const config = { url: data.url, headers: data.headers };
  if (editingIntegration.value) {
    intStore.editIntegration(
      editingIntegration.value.id,
      data.name,
      config,
      data.events,
      editingIntegration.value.enabled,
    );
  } else {
    intStore.addIntegration("webhook", data.name, config, data.events);
  }
  closeConfig();
}

function handleSlackSave(data: {
  name: string;
  botToken: string;
  statusEmoji: string;
  events: string[];
}) {
  const config = { botToken: data.botToken, statusEmoji: data.statusEmoji };
  if (editingIntegration.value) {
    intStore.editIntegration(
      editingIntegration.value.id,
      data.name,
      config,
      data.events,
      editingIntegration.value.enabled,
    );
  } else {
    intStore.addIntegration("slack", data.name, config, data.events);
  }
  closeConfig();
}

function handleDiscordSave(data: { name: string; webhookUrl: string; events: string[] }) {
  const config = { webhookUrl: data.webhookUrl };
  if (editingIntegration.value) {
    intStore.editIntegration(
      editingIntegration.value.id,
      data.name,
      config,
      data.events,
      editingIntegration.value.enabled,
    );
  } else {
    intStore.addIntegration("discord", data.name, config, data.events);
  }
  closeConfig();
}

function confirmDelete(integration: IntegrationConfig) {
  deleteTarget.value = integration;
  confirmDeleteVisible.value = true;
}

function handleDelete() {
  if (deleteTarget.value) {
    intStore.removeIntegration(deleteTarget.value.id);
  }
  confirmDeleteVisible.value = false;
  deleteTarget.value = null;
}

function cancelDelete() {
  confirmDeleteVisible.value = false;
  deleteTarget.value = null;
}

function getLastEvent(integrationId: string): EventLogEntry | undefined {
  return intStore.eventLog.find((e) => e.integration_id === integrationId);
}
</script>

<template>
  <div class="settings-view">
    <h1 class="title">Settings</h1>

    <div class="sections">
      <!-- Timer Section -->
      <section class="section">
        <h2 class="section-title">Timer</h2>

        <div class="field">
          <label class="field-label">Focus Duration (min)</label>
          <input
            type="number"
            class="field-input number-input"
            :value="store.settings.work_duration"
            min="1"
            max="120"
            @change="onNumberChange('work_duration', $event)"
          />
        </div>

        <div class="field">
          <label class="field-label">Short Break (min)</label>
          <input
            type="number"
            class="field-input number-input"
            :value="store.settings.short_break_duration"
            min="1"
            max="60"
            @change="onNumberChange('short_break_duration', $event)"
          />
        </div>

        <div class="field">
          <label class="field-label">Long Break (min)</label>
          <input
            type="number"
            class="field-input number-input"
            :value="store.settings.long_break_duration"
            min="1"
            max="60"
            @change="onNumberChange('long_break_duration', $event)"
          />
        </div>

        <div class="field">
          <label class="field-label">Cycles before long break</label>
          <input
            type="number"
            class="field-input number-input"
            :value="store.settings.cycles_before_long_break"
            min="1"
            max="10"
            @change="onNumberChange('cycles_before_long_break', $event)"
          />
        </div>
      </section>

      <!-- Automation Section -->
      <section class="section">
        <h2 class="section-title">Automation</h2>

        <div class="field row">
          <label class="field-label">Auto-start focus</label>
          <button
            class="toggle"
            :class="{ active: store.settings.auto_start_work }"
            @click="onToggle('auto_start_work')"
          >
            <span class="toggle-knob" />
          </button>
        </div>

        <div class="field row">
          <label class="field-label">Auto-start breaks</label>
          <button
            class="toggle"
            :class="{ active: store.settings.auto_start_break }"
            @click="onToggle('auto_start_break')"
          >
            <span class="toggle-knob" />
          </button>
        </div>

        <div class="field row">
          <label class="field-label">Launch at login</label>
          <button
            class="toggle"
            :class="{ active: store.settings.auto_launch }"
            @click="onToggle('auto_launch')"
          >
            <span class="toggle-knob" />
          </button>
        </div>
      </section>

      <!-- Sound Section -->
      <section class="section">
        <h2 class="section-title">Sound</h2>

        <div class="field row">
          <label class="field-label">Alarm sound</label>
          <button
            class="toggle"
            :class="{ active: store.settings.sound_enabled }"
            @click="onToggle('sound_enabled')"
          >
            <span class="toggle-knob" />
          </button>
        </div>

        <div v-if="store.settings.sound_enabled" class="field">
          <label class="field-label">Volume</label>
          <div class="range-row">
            <input
              type="range"
              class="range-input"
              :value="store.settings.sound_volume"
              min="0"
              max="100"
              @input="onRangeChange('sound_volume', $event)"
            />
            <span class="range-value">{{ store.settings.sound_volume }}%</span>
          </div>
        </div>
      </section>

      <!-- Focus Section -->
      <section class="section">
        <h2 class="section-title">Focus Mode</h2>

        <div class="field row">
          <label class="field-label">
            <span>Do Not Disturb</span>
            <span class="field-hint">Silence notifications during focus</span>
          </label>
          <button
            class="toggle"
            :class="{ active: store.settings.dnd_enabled }"
            @click="onToggle('dnd_enabled')"
          >
            <span class="toggle-knob" />
          </button>
        </div>
      </section>

      <!-- Focus Sounds Section -->
      <section class="section">
        <h2 class="section-title">Focus Sounds</h2>
        <SoundPresets />
        <SoundMixer />
      </section>

      <!-- AI Coach Section -->
      <section class="section">
        <h2 class="section-title">AI Coach</h2>
        <AiSettings />
      </section>

      <!-- Integrations Section -->
      <section class="section">
        <div class="section-header-row">
          <h2 class="section-title">Integrations</h2>
          <div class="add-dropdown-wrap">
            <button class="add-btn" type="button" @click="openAddDropdown">+ Add</button>
            <Transition name="dropdown">
              <div v-if="addDropdownOpen" class="add-dropdown">
                <button class="add-dropdown-item" type="button" @click="startAdd('webhook')">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    width="14"
                    height="14"
                  >
                    <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" />
                    <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />
                  </svg>
                  Webhook
                </button>
                <button class="add-dropdown-item" type="button" @click="startAdd('slack')">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    width="14"
                    height="14"
                  >
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                  </svg>
                  Slack
                </button>
                <button class="add-dropdown-item" type="button" @click="startAdd('discord')">
                  <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    width="14"
                    height="14"
                  >
                    <circle cx="12" cy="12" r="10" />
                    <path d="M8 14s1.5 2 4 2 4-2 4-2" />
                    <line x1="9" y1="9" x2="9.01" y2="9" />
                    <line x1="15" y1="9" x2="15.01" y2="9" />
                  </svg>
                  Discord
                </button>
              </div>
            </Transition>
          </div>
        </div>

        <div v-if="intStore.integrations.length === 0" class="integrations-empty">
          No integrations configured. Add one to connect with external services.
        </div>

        <IntegrationCard
          v-for="integration in intStore.integrations"
          :key="integration.id"
          :integration="integration"
          :testing="intStore.testing === integration.id"
          :last-event="getLastEvent(integration.id)"
          @edit="startEdit(integration)"
          @delete="confirmDelete(integration)"
          @test="intStore.testIntegration(integration.id)"
          @toggle="intStore.toggleIntegration(integration.id)"
        />

        <EventLog v-if="intStore.integrations.length > 0" :limit="10" />
      </section>

      <!-- Config Modals -->
      <WebhookConfig
        :visible="configVisible && configType === 'webhook'"
        :integration="editingIntegration"
        @save="handleWebhookSave"
        @cancel="closeConfig"
      />
      <SlackConfig
        :visible="configVisible && configType === 'slack'"
        :integration="editingIntegration"
        @save="handleSlackSave"
        @cancel="closeConfig"
      />
      <DiscordConfig
        :visible="configVisible && configType === 'discord'"
        :integration="editingIntegration"
        @save="handleDiscordSave"
        @cancel="closeConfig"
      />

      <ConfirmDialog
        :visible="confirmDeleteVisible"
        title="Delete Integration"
        :message="`Are you sure you want to delete '${deleteTarget?.name ?? ''}'? This action cannot be undone.`"
        confirm-label="Delete"
        confirm-variant="danger"
        @confirm="handleDelete"
        @cancel="cancelDelete"
      />

      <!-- Reset -->
      <section class="section">
        <button class="reset-btn" @click="store.resetToDefaults()">Reset to defaults</button>
      </section>
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  padding: 24px 20px 40px;
  max-width: 560px;
  height: 100%;
  overflow-y: auto;
}

.title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: 24px;
}

.sections {
  display: flex;
  flex-direction: column;
  gap: 28px;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted);
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-subtle);
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

.field-hint {
  font-size: 12px;
  color: var(--text-muted);
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
}

.field-input:focus {
  border-color: var(--color-work);
}

.number-input {
  width: 80px;
}

/* Toggle switch */
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
  transition:
    background 0.2s,
    box-shadow 0.2s;
}

.toggle:hover {
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.06);
}

.toggle:active {
  transform: scale(0.95);
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
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.toggle.active .toggle-knob {
  transform: translateX(20px);
}

/* Range */
.range-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.range-input {
  flex: 1;
  -webkit-appearance: none;
  appearance: none;
  height: 4px;
  background: var(--bg-hover);
  border-radius: 2px;
  outline: none;
}

.range-input::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: var(--color-work);
  cursor: pointer;
}

.range-value {
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--text-muted);
  min-width: 36px;
  text-align: right;
}

/* Reset button */
.reset-btn {
  background: none;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  font-size: 13px;
  padding: 8px 16px;
  cursor: pointer;
  transition:
    color 0.15s,
    border-color 0.15s;
  align-self: flex-start;
}

.reset-btn:hover {
  color: var(--color-priority-high);
  border-color: var(--color-priority-high);
}

.reset-btn:active {
  transform: scale(0.97);
}

/* Integrations */
.section-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-subtle);
}

.section-header-row .section-title {
  margin: 0;
  padding: 0;
  border: none;
}

.add-dropdown-wrap {
  position: relative;
}

.add-btn {
  padding: 4px 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  background: var(--bg-card);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition:
    color 0.15s,
    border-color 0.15s;
}

.add-btn:hover {
  color: var(--text-primary);
  border-color: var(--text-muted);
}

.add-btn:active {
  transform: scale(0.95);
}

.add-dropdown {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  min-width: 160px;
  background: var(--bg-card);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-dropdown);
  z-index: 50;
  overflow: hidden;
}

.add-dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  text-align: left;
  transition:
    background 0.1s,
    color 0.1s;
}

.add-dropdown-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

.integrations-empty {
  font-size: 13px;
  color: var(--text-muted);
  padding: 8px 0;
}

/* Dropdown transition */
.dropdown-enter-active {
  transition:
    opacity 0.15s ease,
    transform 0.15s ease;
}

.dropdown-leave-active {
  transition:
    opacity 0.1s ease,
    transform 0.1s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px) scale(0.97);
}
</style>
