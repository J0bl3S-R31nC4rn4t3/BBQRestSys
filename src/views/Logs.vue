<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { logService, type SystemLog } from '../services/logService';
import Header from '../components/layout/Header.vue';
import DataLoader from '../components/ui/DataLoader.vue';
import BaseButton from '../components/ui/BaseButton.vue';
import BaseBadge from '../components/ui/BaseBadge.vue';
import ViewLogModal from '../components/modal/ViewLogModal.vue';
import { useResponsive } from '../composables/useResponsive';

const { fontSm, fontBase, isMobile } = useResponsive();

const logs = ref<SystemLog[]>([]);
const isLoadingData = ref(true);
const filterCategory = ref('All');

const isLogModalOpen = ref(false);
const selectedLog = ref<SystemLog | null>(null);

async function loadLogs() {
  isLoadingData.value = true;
  try {
    logs.value = await logService.getRecentLogs(200);
  } catch (error) {
    console.error("Failed to load system logs:", error);
  } finally {
    setTimeout(() => {
      isLoadingData.value = false;
    }, 600);
  }
}

const availableCategories = computed(() => {
  const categories = new Set(logs.value.map(log => log.log_category));
  return ['All', ...Array.from(categories)];
});

const filteredLogs = computed(() => {
  if (filterCategory.value === 'All') return logs.value;
  return logs.value.filter(log => log.log_category === filterCategory.value);
});

function formatTime(timestampStr: string) {
  const date = new Date(timestampStr);

  if (isNaN(date.getTime())) {
    return 'Invalid Date';
  }

  return date.toLocaleString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit'
  });
}

function getCategoryVariant(category: string): 'info' | 'warning' | 'success' | 'danger' | 'default' {
  if (category === 'POS') return 'info';
  if (category === 'PREP') return 'warning';
  if (category === 'INVENTORY') return 'success';
  if (category === 'ADMIN') return 'danger';
  return 'default';
}

function openLogDetails(log: SystemLog) {
  selectedLog.value = log;
  isLogModalOpen.value = true;
}

function closeLogDetails() {
  isLogModalOpen.value = false;
  selectedLog.value = null;
}

onMounted(() => {
  loadLogs();
});
</script>

<template>
  <div class="h-full flex flex-col relative">
    <div class="bg-surface-container-low p-3 rounded-2xl shadow-sm border border-outline-variant/15 flex-1 flex flex-col overflow-hidden">

      <Header 
        title="System Logs" 
        subtitle="Immutable audit trail of all system transactions."
        isSticky
        isGlass
        customClass="-mt-3 md:-mt-4 -mx-3 md:-mx-4 px-3 md:px-4 pt-3 md:pt-4 pb-4 mb-6 border-b border-outline-variant/20 rounded-t-2xl"
      >
        <template #actions>
          <div :class="['flex items-center gap-3 w-full md:w-auto', isMobile ? 'flex-col items-stretch' : '']">
            <label :class="['font-bold text-on-surface-variant uppercase tracking-widest text-[10px]', fontSm, isMobile ? 'hidden' : 'block']">Filter by Event:</label>
            <select v-model="filterCategory"
              :class="['bg-surface-container border border-outline-variant/30 text-on-surface focus:outline-none focus:border-primary-container focus:ring-1 focus:ring-primary-container rounded-lg px-4 py-2 font-bold uppercase tracking-wider text-xs w-full md:w-auto transition-colors']">
              <option v-for="category in availableCategories" :key="category" :value="category">
                {{ category === 'All' ? 'All Events' : category }}
              </option>
            </select>
            <BaseButton variant="secondary" @click="loadLogs" title="Refresh Logs" class="px-4 py-2">
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                  d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15">
                </path>
              </svg>
            </BaseButton>
          </div>
        </template>
      </Header>

      <DataLoader v-if="isLoadingData" message="Retrieving secure audit trails..." />

      <div v-else class="flex-1 overflow-auto border border-outline-variant/15 rounded-xl bg-surface-container-low">
        <table class="w-full text-left border-collapse">
          <thead class="bg-surface-container sticky top-0 z-10">
            <tr :class="['border-b border-outline-variant/20 text-on-surface-variant uppercase tracking-widest font-bold', fontSm]">
              <th class="p-4 w-48">Timestamp</th>
              <th class="p-4 w-32 hidden md:table-cell">Category</th>
              <th class="p-4 w-48 hidden md:table-cell">User</th>
              <th class="p-4">Action Description</th>
              <th class="p-4 text-right hidden md:table-cell">Details</th>
            </tr>
          </thead>
          <tbody class="text-on-surface">
            <tr v-if="filteredLogs.length === 0">
              <td colspan="5" class="py-12 text-center text-on-surface-variant font-bold uppercase tracking-widest text-sm">
                No system activity recorded yet.
              </td>
            </tr>
            <tr v-for="log in filteredLogs" :key="log.log_id" @click="openLogDetails(log)"
              class="border-b border-outline-variant/10 hover:bg-surface-container-high transition-colors cursor-pointer group"
              title="Click to view full details">
              <td :class="['p-4 whitespace-nowrap font-bold text-on-surface', fontSm]">{{ formatTime(log.timestamp) }}</td>
              <td class="p-4 hidden md:table-cell">
                <BaseBadge :text="log.log_category" :variant="getCategoryVariant(log.log_category)" />
              </td>
              <td :class="['p-4 font-black hidden md:table-cell text-on-surface', fontBase]">{{ log.staff_name || 'System Admin' }}</td>
              <td :class="['p-4 font-medium group-hover:text-primary-container transition-colors', fontBase]">
                <span class="block md:hidden text-[10px] text-on-surface-variant font-bold uppercase tracking-widest mb-1">{{ log.log_category }} • {{ log.staff_name || 'Admin' }}</span>
                {{ log.description }}
              </td>
              <td :class="['p-4 text-on-surface-variant font-mono text-right truncate max-w-50 hidden md:table-cell', fontSm]">{{ log.details || '-' }}</td>
            </tr>
          </tbody>
        </table>
      </div>

    </div>

    <ViewLogModal :is-open="isLogModalOpen" :log="selectedLog" @close="closeLogDetails" />

  </div>
</template>