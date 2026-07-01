<script setup lang="ts">
import { computed, ref, watch, onBeforeUnmount } from 'vue';
import BaseButton from '../ui/BaseButton.vue';
import { useResponsive } from '../../composables/useResponsive';
import type { ActiveOrder } from '../../services/posService';

const { fontLg } = useResponsive();

const props = defineProps<{
  isOpen: boolean;
  order: ActiveOrder | null;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'print'): void;
}>();

// --- Dragging Logic ---
const position = ref({ x: 0, y: 0 });
const isDragging = ref(false);
let startMouse = { x: 0, y: 0 };
let startPosition = { x: 0, y: 0 };

function startDrag(event: MouseEvent) {
  if ((event.target as HTMLElement).closest('button')) return;

  isDragging.value = true;
  startMouse = { x: event.clientX, y: event.clientY };
  startPosition = { x: position.value.x, y: position.value.y };
  
  document.addEventListener('mousemove', onDrag);
  document.addEventListener('mouseup', stopDrag);
}

function onDrag(event: MouseEvent) {
  if (!isDragging.value) return;
  const dx = event.clientX - startMouse.x;
  const dy = event.clientY - startMouse.y;
  position.value = { x: startPosition.x + dx, y: startPosition.y + dy };
}

function stopDrag() {
  isDragging.value = false;
  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
}

watch(() => props.isOpen, (newVal) => {
  if (!newVal) {
    position.value = { x: 0, y: 0 };
  }
});

onBeforeUnmount(() => {
  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
});
// ----------------------

const totalItems = computed(() => {
  if (!props.order || !props.order.cart_items) return 0;
  return props.order.cart_items.reduce((sum, item) => sum + item.qty, 0);
});

// Manual formatter strictly outputs exactly 19 chars: "MM/DD/YYYY HH:MM AM/PM"
const formattedDate = computed(() => {
  if (!props.order?.timestamp) return '';
  
  const d = new Date(props.order.timestamp);
  const month = (d.getMonth() + 1).toString().padStart(2, '0');
  const day = d.getDate().toString().padStart(2, '0');
  const year = d.getFullYear();
  
  let hours = d.getHours();
  const minutes = d.getMinutes().toString().padStart(2, '0');
  const ampm = hours >= 12 ? 'PM' : 'AM';
  hours = hours % 12;
  hours = hours ? hours : 12; 
  const strHours = hours.toString().padStart(2, '0');

  return `${month}/${day}/${year} ${strHours}:${minutes} ${ampm}`;
});
</script>

<template>
  <div v-if="isOpen && order" class="fixed inset-0 z-[120] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm transition-opacity" @click.self="$emit('close')">
    
    <div 
      :style="{ transform: `translate(${position.x}px, ${position.y}px)` }"
      class="bg-surface rounded-2xl w-full max-w-sm max-h-[95vh] shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-200 border border-outline-variant/20 will-change-transform"
    >
      
      <div 
        @mousedown.prevent="startDrag"
        :class="[
          'px-5 py-4 border-b border-outline-variant/20 flex justify-between items-center bg-surface-container-low shrink-0 select-none',
          isDragging ? 'cursor-grabbing' : 'cursor-grab'
        ]"
      >
        <h3 :class="['font-black text-on-surface tracking-tight', fontLg]">Receipt Preview</h3>
        <button @click="$emit('close')" class="text-on-surface-variant hover:text-error transition-colors active:scale-90 z-10 cursor-pointer">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      </div>

      <div class="p-6 bg-surface-container-lowest overflow-y-auto flex-1 flex justify-center">
        <div class="bg-white text-black p-5 shadow-md w-full max-w-[280px] h-fit font-mono text-xs leading-tight rounded-sm relative mx-auto">
          <div class="absolute -top-1 left-0 w-full h-2 bg-repeat-x" style="background-image: radial-gradient(circle at 4px 0px, transparent 4px, white 5px); background-size: 8px 10px;"></div>
          
          <div class="text-center mb-4">
            <h2 class="font-bold text-sm mb-1">BBQ NA MURAG LAMI</h2>
            <p class="text-[10px] text-gray-700">Ampayon, Butuan City</p>
          </div>

          <div class="mb-3 pb-3 text-gray-700 space-y-1">
            <p class="flex justify-between"><span>Order #:</span> <span class="font-bold">{{ order.order_id }}</span></p>
            <p class="flex justify-between"><span>Type:</span> <span>{{ order.order_type }}</span></p>
            <p class="flex justify-between"><span>Date:</span> <span>{{ formattedDate }}</span></p>
            <p class="mt-3 pt-2">Customer/Table:</p>
            <p class="font-bold truncate">{{ order.customer_identifier }}</p>
          </div>

          <div class="border-y border-dashed border-gray-400 py-1 mb-2 flex justify-between font-bold">
            <span class="w-6">QTY</span>
            <span class="flex-1 pl-1">ITEM</span>
            <span class="w-12 text-right">AMT</span>
          </div>

          <div class="flex flex-col gap-1 mb-3">
            <div v-for="(item, idx) in order.cart_items" :key="idx" class="flex justify-between items-start">
              <span class="w-6">{{ item.qty }}</span>
              <span class="flex-1 pl-1 pr-2 truncate">{{ item.pos_display_name }}</span>
              <span class="w-12 text-right">{{ (item.unit_price * item.qty).toFixed(2) }}</span>
            </div>
          </div>

          <div class="border-t border-dashed border-gray-400 pt-2 mb-4">
            <div class="flex justify-between items-center font-bold text-sm">
              <span>TOTAL</span>
              <span>PHP {{ order.total_amount.toFixed(2) }}</span>
            </div>
            <p class="text-right text-[10px] text-gray-500 mt-1">Item Count: {{ totalItems }}</p>
          </div>

          <div class="text-center text-[10px] text-gray-700 space-y-1 pb-4">
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
        <BaseButton variant="primary" @click="$emit('print')" class="flex-1 py-2.5 flex justify-center items-center gap-2 shadow-sm">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"></path></svg>
          <span class="font-bold uppercase tracking-widest text-sm">Print Receipt</span>
        </BaseButton>
      </div>

    </div>
  </div>
</template>