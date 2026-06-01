<script setup lang="ts">
import BaseButton from '../ui/BaseButton.vue';
import { useResponsive } from '../../composables/useResponsive';

const { fontSm, fontLg } = useResponsive();

defineProps<{
  isOpen: boolean;
  customerIdentifier: string;
}>();

defineEmits<{
  (e: 'print'): void;
  (e: 'view'): void;
  (e: 'close'): void;
}>();
</script>

<template>
  <div v-if="isOpen" class="fixed inset-0 z-[130] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm transition-opacity" @click.self="$emit('close')">
    <div class="bg-surface rounded-2xl w-full max-w-sm shadow-2xl flex flex-col overflow-hidden text-center p-6 border border-outline-variant/20 animate-in zoom-in-95 duration-200">
      
      <div class="mx-auto flex items-center justify-center h-14 w-14 rounded-full bg-success/20 text-success mb-4 border border-success/30">
        <svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7"></path></svg>
      </div>
      
      <h3 :class="['font-black text-on-surface mb-2', fontLg]">Payment Settled</h3>
      <p :class="['text-on-surface-variant mb-6', fontSm]">
        Would you like to print a receipt for <br>
        <span class="font-bold text-on-surface">{{ customerIdentifier }}</span>?
      </p>

      <div class="flex flex-col gap-3">
        <BaseButton variant="primary" @click="$emit('print')" class="w-full py-3 shadow-sm">
          Print Receipt
        </BaseButton>
        <BaseButton variant="secondary" @click="$emit('view')" class="w-full py-3">
          View Digital Receipt
        </BaseButton>
        <button @click="$emit('close')" class="text-on-surface-variant hover:text-on-surface text-sm font-bold mt-2 py-2 transition-colors active:scale-95">
          No Thanks, Close
        </button>
      </div>

    </div>
  </div>
</template>