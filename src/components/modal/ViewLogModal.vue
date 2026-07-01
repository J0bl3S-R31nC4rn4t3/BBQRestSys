<script setup lang="ts">
import type { SystemLog } from '../../services/logService';
import BaseBadge from '../ui/BaseBadge.vue';

// Define props without assigning to an unused variable
defineProps<{
  isOpen: boolean;
  log: SystemLog | null;
}>();

defineEmits<{
  close: [];
}>();

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

function getCategoryVariant(category: string | undefined): 'info' | 'warning' | 'success' | 'danger' | 'default' {
  if (!category) return 'default';
  if (category === 'POS') return 'info';
  if (category === 'PREP') return 'warning';
  if (category === 'INVENTORY') return 'success';
  if (category === 'ADMIN') return 'danger';
  return 'default';
}
</script>

<template>
  <div v-if="isOpen && log" class="fixed inset-0 bg-black/60 z-50 flex items-center justify-center backdrop-blur-md" @click.self="$emit('close')">
    <div class="bg-surface-container-low border border-outline-variant/20 rounded-2xl shadow-2xl w-full max-w-lg p-0 m-4 overflow-hidden flex flex-col max-h-[90vh]">
      
      <div class="px-6 py-5 bg-surface-container-highest/30 border-b border-outline-variant/10 flex justify-between items-center">
        <div>
          <h3 class="text-xl font-black text-on-surface tracking-tight">Log Details</h3>
          <p class="text-xs text-on-surface-variant font-mono mt-1 tracking-wider uppercase">ID: {{ log.log_id }}</p>
        </div>
        <button type="button" @click="$emit('close')" class="text-on-surface-variant hover:text-on-surface bg-surface-container hover:bg-surface-container-high p-2 rounded-full transition-colors active:scale-90">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
        </button>
      </div>
      
      <div class="p-6 overflow-y-auto flex-1 space-y-6 bg-surface">
        
        <div class="flex items-center justify-between">
          <div>
            <p class="text-xs text-on-surface-variant font-bold uppercase tracking-widest mb-2">Category</p>
            <BaseBadge :text="log.log_category" :variant="getCategoryVariant(log.log_category)" />
          </div>
          <div class="text-right">
            <p class="text-xs text-on-surface-variant font-bold uppercase tracking-widest mb-2">Timestamp</p>
            <p class="text-sm font-black text-on-surface">{{ formatTime(log.timestamp) }}</p>
          </div>
        </div>

        <div class="border-t border-outline-variant/15 pt-5">
          <p class="text-xs text-on-surface-variant font-bold uppercase tracking-widest mb-3">Initiated By</p>
          <div class="flex items-center gap-4 bg-surface-container p-3 rounded-xl border border-outline-variant/10">
            <div class="w-10 h-10 rounded-full bg-primary-container/20 text-primary flex items-center justify-center font-black text-sm border border-primary-container/30 shadow-sm">
              {{ log.staff_name ? log.staff_name.charAt(0) : 'S' }}
            </div>
            <div>
              <p class="font-bold text-on-surface">{{ log.staff_name || 'System Admin' }}</p>
              <p class="text-xs text-on-surface-variant tracking-wider uppercase mt-0.5">Staff ID: {{ log.staff_id || 'N/A' }}</p>
            </div>
          </div>
        </div>

        <div class="border-t border-outline-variant/15 pt-5">
          <p class="text-xs text-on-surface-variant font-bold uppercase tracking-widest mb-3">Action Description</p>
          <p class="text-base text-on-surface font-medium bg-surface-container-low p-4 rounded-xl border border-outline-variant/10">{{ log.description }}</p>
        </div>

        <div class="border-t border-outline-variant/15 pt-5">
          <p class="text-xs text-on-surface-variant font-bold uppercase tracking-widest mb-3">Extended Details / Data</p>
          <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 shadow-inner">
             <p class="text-sm text-on-surface font-mono whitespace-pre-wrap wrap-break-word leading-relaxed">
              {{ log.details || 'No extended details recorded for this event.' }}
            </p>
          </div>
        </div>

      </div>

    </div>
  </div>
</template>