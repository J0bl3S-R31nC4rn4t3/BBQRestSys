<script setup lang="ts">
import { computed } from 'vue';
import BaseButton from '../ui/BaseButton.vue';
import { useResponsive } from '../../composables/useResponsive';
import type { ActiveOrder } from '../../services/posService';

const { fontSm, fontBase, fontLg, fontXl } = useResponsive();

const props = defineProps<{
  isOpen: boolean;
  order: ActiveOrder | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'print', orderId: number): void;
}>();

// Calculate the total number of items
const totalItems = computed(() => {
  if (!props.order || !props.order.cart_items) return 0;
  return props.order.cart_items.reduce((sum, item) => sum + item.qty, 0);
});

// Format the date for the receipt
const formattedDate = computed(() => {
  if (!props.order) return '';
  const date = new Date(props.order.timestamp);
  return date.toLocaleString('en-PH', {
    month: 'short',
    day: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
});
</script>

<template>
  <div v-if="isOpen && order" class="fixed inset-0 z-[120] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm transition-opacity" @click.self="$emit('close')">
    <div class="bg-surface rounded-2xl w-full max-w-sm shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-200 border border-outline-variant/20">
      
      <div class="px-5 py-4 border-b border-outline-variant/20 flex justify-between items-center bg-surface-container-low shrink-0">
        <h3 :class="['font-black text-on-surface tracking-tight', fontLg]">Receipt Preview</h3>
        <button @click="$emit('close')" class="text-on-surface-variant hover:text-error transition-colors active:scale-90">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      </div>

      <div class="p-6 bg-surface-container-lowest overflow-y-auto flex-1 flex justify-center">
        <div class="bg-white text-black p-6 shadow-md w-full max-w-[320px] font-mono text-sm leading-tight rounded-sm relative">
          <div class="absolute -top-1 left-0 w-full h-2 bg-repeat-x" style="background-image: radial-gradient(circle at 4px 0px, transparent 4px, white 5px); background-size: 8px 10px;"></div>
          
          <div class="text-center mb-4">
            <h2 class="font-bold text-lg mb-1">BBQ NA MURAG LAMI</h2>
            <p class="text-xs text-gray-700">Cagayan De Oro City</p>
          </div>

          <div class="border-b-2 border-dashed border-gray-400 mb-3 pb-3 text-xs text-gray-700 space-y-1">
            <p class="flex justify-between"><span>Order #:</span> <span class="font-bold">{{ order.order_id }}</span></p>
            <p class="flex justify-between"><span>Type:</span> <span>{{ order.order_type }}</span></p>
            <p class="flex justify-between"><span>Date:</span> <span>{{ formattedDate }}</span></p>
            <p class="flex justify-between mt-2 pt-2 border-t border-gray-200"><span>Customer/Table:</span> <span class="font-bold">{{ order.customer_identifier }}</span></p>
          </div>

          <table class="w-full text-xs mb-3">
            <thead>
              <tr class="border-b-2 border-dashed border-gray-400">
                <th class="text-left pb-1 font-normal">QTY</th>
                <th class="text-left pb-1 font-normal pl-2">ITEM</th>
                <th class="text-right pb-1 font-normal">AMT</th>
              </tr>
            </thead>
            <tbody class="align-top">
              <tr v-for="(item, idx) in order.cart_items" :key="idx">
                <td class="py-1 pt-2">{{ item.qty }}</td>
                <td class="py-1 pt-2 pl-2 pr-1">{{ item.pos_display_name }}</td>
                <td class="py-1 pt-2 text-right">{{ (item.unit_price * item.qty).toFixed(2) }}</td>
              </tr>
            </tbody>
          </table>

          <div class="border-t-2 border-dashed border-gray-400 pt-3 mb-6">
            <div class="flex justify-between items-center text-sm font-bold">
              <span>TOTAL</span>
              <span class="text-lg">PHP {{ order.total_amount.toFixed(2) }}</span>
            </div>
            <p class="text-right text-xs text-gray-500 mt-1">Item Count: {{ totalItems }}</p>
          </div>

          <div class="text-center text-xs text-gray-700 space-y-1 pb-4">
            <p>Thank you for dining with us!</p>
            <p>Please come again.</p>
          </div>
          
           <div class="absolute -bottom-1 left-0 w-full h-2 bg-repeat-x" style="background-image: radial-gradient(circle at 4px 8px, transparent 4px, white 5px); background-size: 8px 10px;"></div>
        </div>
      </div>

      <div class="p-4 bg-surface-container-low border-t border-outline-variant/20 flex gap-3 shrink-0">
        <BaseButton variant="secondary" @click="$emit('close')" class="flex-1 py-2.5">
          Close
        </BaseButton>
        <BaseButton variant="primary" @click="$emit('print', order.order_id)" class="flex-1 py-2.5 flex justify-center items-center gap-2 shadow-sm">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"></path></svg>
          <span class="font-bold uppercase tracking-widest text-sm">Print Receipt</span>
        </BaseButton>
      </div>

    </div>
  </div>
</template>