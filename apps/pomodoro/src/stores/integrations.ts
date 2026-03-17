import { defineStore } from "pinia";
import { computed, ref } from "vue";
import {
  createIntegration,
  updateIntegration,
  deleteIntegration,
  listIntegrations,
  testIntegration as testIntegrationIpc,
  getEventLog,
  type IntegrationConfig,
  type EventLogEntry,
} from "@/lib/tauri";
import { useToast } from "@/composables/useToast";

export const useIntegrationsStore = defineStore("integrations", () => {
  const integrations = ref<IntegrationConfig[]>([]);
  const eventLog = ref<EventLogEntry[]>([]);
  const loading = ref(false);
  const testing = ref<string | null>(null);

  const { showToast } = useToast();

  const enabledCount = computed(() => integrations.value.filter((i) => i.enabled).length);

  async function fetchIntegrations() {
    loading.value = true;
    try {
      integrations.value = await listIntegrations();
    } catch {
      showToast("Failed to load integrations", "error");
    } finally {
      loading.value = false;
    }
  }

  async function fetchEventLog(integrationId?: string, limit = 20) {
    try {
      eventLog.value = await getEventLog(integrationId, limit);
    } catch {
      showToast("Failed to load event log", "error");
    }
  }

  async function addIntegration(
    type: string,
    name: string,
    config: Record<string, unknown>,
    events: string[],
  ) {
    try {
      const created = await createIntegration(
        type,
        name,
        JSON.stringify(config),
        JSON.stringify(events),
      );
      integrations.value.push(created);
      showToast(`Integration "${name}" created`, "success");
    } catch {
      showToast("Failed to create integration", "error");
    }
  }

  async function editIntegration(
    id: string,
    name: string,
    config: Record<string, unknown>,
    events: string[],
    enabled: boolean,
  ) {
    try {
      await updateIntegration(id, name, JSON.stringify(config), JSON.stringify(events), enabled);
      const idx = integrations.value.findIndex((i) => i.id === id);
      if (idx !== -1) {
        integrations.value[idx] = {
          ...integrations.value[idx],
          name,
          config: JSON.stringify(config),
          events: JSON.stringify(events),
          enabled,
          updated_at: new Date().toISOString(),
        };
      }
      showToast(`Integration "${name}" updated`, "success");
    } catch {
      showToast("Failed to update integration", "error");
    }
  }

  async function removeIntegration(id: string) {
    try {
      await deleteIntegration(id);
      integrations.value = integrations.value.filter((i) => i.id !== id);
      showToast("Integration deleted", "success");
    } catch {
      showToast("Failed to delete integration", "error");
    }
  }

  async function toggleIntegration(id: string) {
    const item = integrations.value.find((i) => i.id === id);
    if (!item) return;
    const config = JSON.parse(item.config) as Record<string, unknown>;
    const events = JSON.parse(item.events) as string[];
    await editIntegration(id, item.name, config, events, !item.enabled);
  }

  async function testIntegration(id: string) {
    testing.value = id;
    try {
      const result = await testIntegrationIpc(id);
      showToast(result || "Test successful", "success");
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      showToast(`Test failed: ${msg}`, "error");
    } finally {
      testing.value = null;
    }
  }

  return {
    integrations,
    eventLog,
    loading,
    testing,
    enabledCount,
    fetchIntegrations,
    fetchEventLog,
    addIntegration,
    editIntegration,
    removeIntegration,
    toggleIntegration,
    testIntegration,
  };
});
