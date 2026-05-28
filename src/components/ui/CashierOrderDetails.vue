<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import BaseButton from './BaseButton.vue';
import { useResponsive } from '../../composables/useResponsive';
import type { ActiveOrder, PosItem, CartItem } from '../../services/posService';
import type { PendingOrder } from '../../services/queueService';

const { fontSm, fontBase, fontLg } = useResponsive();

const props = defineProps<{
  selectedOrder: ActiveOrder | null;
  selectedPending: PendingOrder | null;
  availableItems: PosItem[];
  tableNumberInput: string;
}>();

const emit = defineEmits<{
  (e: 'update:tableNumberInput', val: string): void;
  (e: 'clear-pending'): void;
  (e: 'reject-pending'): void;
  (e: 'accept-pending'): void;
  (e: 'update-status', status: string): void;
  (e: 'settle-payment'): void;
  (e: 'save-pending-edit', updatedData: PendingOrder): void;
  (e: 'preview-receipt'): void; // NEW: Added preview event
}>();

const isEditing = ref(false);
const editCart = ref<CartItem[]>([]);
const newItemSelection = ref<number | ''>('');
const newItemQty = ref<number>(1);

watch(() => props.selectedOrder, () => { isEditing.value = false; });
watch(() => props.selectedPending, () => { isEditing.value = false; });

function validateTableInput(e: Event) {
  const target = e.target as HTMLInputElement;
  let val = target.value.replace(/\D/g, '');
  if (val !== '') {
    let num = parseInt(val, 10);
    if (num > 100) val = '100';
  }
  target.value = val;
  emit('update:tableNumberInput', val);
}

function startEdit() {
  if (props.selectedPending) {
    editCart.value = JSON.parse(JSON.stringify(props.selectedPending.cart_items || []));
    isEditing.value = true;
  }
}

function addEditItem() {
  if (!newItemSelection.value || newItemQty.value <= 0) return;
  const item = props.availableItems.find(i => i.prep_item_id === newItemSelection.value);
  if (item) {
    const existing = editCart.value.find(i => i.prep_item_id === item.prep_item_id);
    if (existing) {
      existing.qty += newItemQty.value;
    } else {
      editCart.value.push({ ...item, qty: newItemQty.value } as CartItem);
    }
  }
  newItemSelection.value = '';
  newItemQty.value = 1;
}

function incrementEditQty(idx: number) {
  editCart.value[idx].qty++;
}

function decrementEditQty(idx: number) {
  if (editCart.value[idx].qty > 1) {
    editCart.value[idx].qty--;
  } else {
    editCart.value.splice(idx, 1);
  }
}

const editTotal = computed(() => editCart.value.reduce((acc, item) => acc + (item.unit_price * item.qty), 0));

function saveEdit() {
  if (props.selectedPending) {
    emit('save-pending-edit', { 
      ...props.selectedPending, 
      cart_items: editCart.value, 
      total: editTotal.value, 
      subtotal: editTotal.value 
    });
  }
  isEditing.value = false;
}
</script>

<template>
  <div class="w-full h-full bg-surface-container-low rounded-xl border border-outline-variant/15 flex flex-col min-h-0 relative overflow-hidden">
    
    <div class="flex-1 p-4 bg-surface flex flex-col min-h-0">
      
      <div v-if="!selectedOrder && !selectedPending" class="h-full flex flex-col items-center justify-center text-on-surface-variant space-y-3 m-auto px-4 text-center">
        <svg class="w-12 h-12 opacity-20" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"></path></svg>
        <p :class="['font-medium', fontBase]">Select an active tab from the right,<br class="hidden sm:block"/>or search a Customer Queue # above.</p>
      </div>

      <div v-else-if="selectedPending" class="h-full max-w-2xl w-full mx-auto flex flex-col gap-1 animate-in fade-in duration-300">
        <div class="bg-surface-container-low border border-outline-variant/15 rounded-xl p-3 sm:p-4 relative flex flex-col flex-1 min-h-0 overflow-hidden">
          <div class="bg-primary-container absolute top-0 left-0 w-full h-1 animate-pulse shadow-[0_0_12px_rgba(255,109,0,0.8)]"></div>
          
          <div class="flex justify-between items-start mb-0 shrink-0 mt-1 gap-2">
            <div class="min-w-0 pr-2">
              <span class="text-primary font-bold tracking-widest uppercase text-[10px]">Kiosk Order Ready</span>
              <h3 :class="['font-black text-on-surface mt-1 truncate', fontLg]">Queue #{{ selectedPending.queue_number }}</h3>
              <p :class="['font-medium text-on-surface-variant truncate', fontSm]">{{ selectedPending.order_type }}</p>
            </div>
            <button v-if="!isEditing" @click="emit('clear-pending')" class="text-on-surface-variant hover:text-on-surface hover:bg-surface-container-high p-2 rounded-full transition-colors active:scale-90 shrink-0">
               <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
            </button>
          </div>

          <div class="flex-1 overflow-y-auto border-t border-b border-outline-variant/10 py-3 my-2 pr-1 sm:pr-2">
            <div class="flex justify-between items-center mb-0 sticky top-0 bg-surface-container-low z-10 pb-2">
              <h4 class="font-bold text-on-surface-variant text-xs uppercase tracking-widest">Customer Cart</h4>
              <button v-if="!isEditing" @click="startEdit" class="text-primary text-xs font-bold hover:underline transition-all">Edit Order</button>
            </div>
            
            <div v-if="!isEditing" class="space-y-2">
              <div v-for="(item, idx) in selectedPending.cart_items" :key="idx" class="flex flex-col xl:flex-row xl:justify-between xl:items-center bg-surface-container py-2 px-3 rounded-lg border border-outline-variant/10 group hover:bg-surface-container-high transition-colors gap-1 xl:gap-0">
                <p :class="['font-bold text-on-surface min-w-0 wrap-break-word', fontSm]"><span class="text-primary-container mr-1">{{ item.qty }}x</span> {{ item.pos_display_name }}</p>
                <span :class="['font-bold text-on-surface-variant group-hover:text-on-surface self-end xl:self-auto shrink-0', fontSm]">₱{{ (item.unit_price * item.qty).toFixed(2) }}</span>
              </div>
            </div>

            <div v-else class="space-y-3">
              <div v-for="(item, idx) in editCart" :key="idx" class="flex flex-col lg:flex-row justify-between items-start lg:items-center bg-surface-container p-3 border border-outline-variant/20 rounded-lg gap-3 lg:gap-0">
                <span class="font-bold text-sm text-on-surface w-full lg:w-auto line-clamp-2 min-w-0 pr-2">{{ item.pos_display_name }}</span>
                <div class="flex items-center justify-between lg:justify-end w-full lg:w-auto gap-4 shrink-0">
                  <span class="font-bold text-on-surface-variant text-sm shrink-0">₱{{ (item.unit_price * item.qty).toFixed(2) }}</span>
                  <div class="flex items-center border border-outline-variant/30 rounded-lg bg-surface-container-high overflow-hidden shrink-0">
                    <button type="button" @click="decrementEditQty(idx)" class="px-4 py-2.5 sm:px-3 sm:py-1.5 text-on-surface hover:bg-surface-variant font-black transition-colors text-lg sm:text-base">-</button>
                    <span class="px-2 py-2.5 sm:py-1.5 text-sm font-bold w-10 sm:w-12 text-center border-l border-r border-outline-variant/30">{{ item.qty }}</span>
                    <button type="button" @click="incrementEditQty(idx)" class="px-4 py-2.5 sm:px-3 sm:py-1.5 text-on-surface hover:bg-surface-variant font-black transition-colors text-lg sm:text-base">+</button>
                  </div>
                </div>
              </div>

              <div class="flex flex-col 2xl:flex-row gap-2 mt-4 bg-surface-container p-3 rounded-lg border border-outline-variant/20 items-stretch 2xl:items-center">
                <select v-model="newItemSelection" class="flex-1 w-full bg-surface-container-low border border-outline-variant/30 text-on-surface rounded px-3 py-2 text-sm focus:outline-none focus:border-primary-container truncate min-w-0">
                  <option value="" disabled>Select Item</option>
                  <option v-for="ai in availableItems" :key="ai.prep_item_id" :value="ai.prep_item_id">{{ ai.pos_display_name }} (₱{{ ai.unit_price }})</option>
                </select>
                <div class="flex gap-2 w-full 2xl:w-auto shrink-0">
                  <input v-model.number="newItemQty" type="number" min="1" class="flex-1 sm:w-16 sm:flex-none bg-surface-container-low border border-outline-variant/30 text-on-surface rounded px-2 py-2 text-sm text-center focus:outline-none focus:border-primary-container">
                  <BaseButton variant="secondary" @click="addEditItem" type="button" class="px-4 py-2 text-sm flex-1 sm:flex-none">Add</BaseButton>
                </div>
              </div>
            </div>
          </div>

          <div class="pt-3 flex justify-between items-center shrink-0 border-t border-outline-variant/10">
            <span :class="['font-bold text-on-surface-variant uppercase tracking-widest text-xs', fontSm]">Total Value</span>
            <span :class="['font-black text-primary', fontLg]">₱{{ isEditing ? editTotal.toFixed(2) : selectedPending.total.toFixed(2) }}</span>
          </div>
        </div>

        <div class="bg-surface-container-low p-3 sm:p-4 rounded-xl border border-outline-variant/15 shrink-0">
          <label :class="['block font-bold text-on-surface-variant uppercase tracking-widest text-xs mb-2', fontSm]">Assign Table Number (1-100)</label>
          <input 
            :value="tableNumberInput" 
            @input="validateTableInput"
            type="text" 
            pattern="[0-9]*"
            placeholder="e.g., 5" 
            class="w-full bg-surface-container text-on-surface placeholder-on-surface-variant/50 border border-outline-variant/30 rounded-lg px-4 py-3 font-bold focus:outline-none focus:ring-1 focus:ring-primary-container focus:border-primary-container transition-all"
          />
        </div>

        <div class="grid grid-cols-2 sm:flex sm:flex-row gap-2 sm:gap-3 shrink-0">
          <template v-if="isEditing">
            <BaseButton variant="secondary" @click="isEditing = false" class="w-full py-3 sm:py-2">Cancel</BaseButton>
            <BaseButton variant="success" @click="saveEdit" class="w-full py-3 sm:py-2">Save</BaseButton>
          </template>
          <template v-else>
            <BaseButton variant="danger" @click="emit('reject-pending')" class="w-full sm:w-auto sm:px-6 py-3 sm:py-2">
              Reject
            </BaseButton>
            <BaseButton variant="primary" @click="emit('accept-pending')" :disabled="!tableNumberInput" class="w-full sm:flex-1 py-3 sm:py-2">
              Accept & Send
            </BaseButton>
          </template>
        </div>
      </div>

      <div v-else-if="selectedOrder" class="h-full max-w-2xl w-full mx-auto flex flex-col gap-4 animate-in fade-in duration-300">
        <div class="bg-surface-container-low border border-outline-variant/15 rounded-xl p-3 sm:p-4 relative flex flex-col flex-1 min-h-0 overflow-hidden">
          <div :class="selectedOrder.status === 'Cooking' ? 'bg-tertiary-container shadow-[0_0_12px_rgba(191,146,76,0.5)]' : 'bg-tertiary shadow-[0_0_12px_rgba(240,190,116,0.5)]'" class="absolute top-0 left-0 w-full h-1"></div>
          
          <div class="flex justify-between items-start mb-0 shrink-0 mt-1 gap-2">
            <div class="min-w-0 pr-2">
              <span class="text-on-surface-variant font-bold tracking-widest uppercase text-[10px]">Order #{{ selectedOrder.order_id }}</span>
              <h3 :class="['font-black text-on-surface mt-1 truncate', fontLg]">{{ selectedOrder.customer_identifier }}</h3>
              <p :class="['font-medium text-on-surface-variant truncate', fontSm]">{{ selectedOrder.order_type }}</p>
            </div>
            
            <div class="flex items-center gap-2 shrink-0">
              <button @click="emit('preview-receipt')" class="text-on-surface-variant hover:text-primary bg-surface-container hover:bg-surface-container-high p-1.5 rounded-full transition-colors active:scale-90" title="Preview Receipt">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path></svg>
              </button>
              
              <span :class="[selectedOrder.status === 'Cooking' ? 'bg-tertiary-container/10 text-tertiary-container border border-tertiary-container/20 animate-pulse' : 'bg-tertiary/10 text-tertiary border border-tertiary/20', 'px-3 py-1 rounded-full text-[10px] font-bold uppercase tracking-wider shrink-0']">
                {{ selectedOrder.status }}
              </span>
            </div>
          </div>

          <div class="flex-1 overflow-y-auto border-t border-b border-outline-variant/10 py-3 my-2 pr-1 sm:pr-2">
            <div class="flex justify-between items-center mb-0 sticky top-0 bg-surface-container-low z-10 pb-2">
              <h4 class="font-bold text-on-surface-variant text-xs uppercase tracking-widest">Order Items</h4>
            </div>
            
            <div v-if="selectedOrder.cart_items && selectedOrder.cart_items.length > 0" class="space-y-2">
              <div v-for="(item, idx) in selectedOrder.cart_items" :key="idx" class="flex flex-col xl:flex-row xl:justify-between xl:items-center bg-surface-container py-2 px-3 rounded-lg border border-outline-variant/10 group hover:bg-surface-container-high transition-colors gap-1 xl:gap-0">
                <p :class="['font-bold text-on-surface min-w-0 wrap-break-word', fontSm]"><span class="text-primary-container mr-1">{{ item.qty }}x</span> {{ item.pos_display_name || 'Item' }}</p>
                <span :class="['font-bold text-on-surface-variant group-hover:text-on-surface self-end xl:self-auto shrink-0', fontSm]">₱{{ (item.unit_price * item.qty).toFixed(2) }}</span>
              </div>
            </div>

            <div v-else class="h-full flex flex-col justify-center items-center py-6">
               <p class="text-on-surface-variant text-sm font-medium text-center">No item details found.</p>
            </div>
          </div>

          <div class="pt-3 flex justify-between items-center shrink-0 border-t border-outline-variant/10">
            <span :class="['font-bold text-on-surface-variant uppercase tracking-widest text-xs', fontSm]">Total Due</span>
            <span :class="['font-black text-primary-container', fontLg]">₱{{ selectedOrder.total_amount.toFixed(2) }}</span>
          </div>
        </div>

        <div class="grid grid-cols-2 sm:flex sm:flex-row gap-2 sm:gap-3 shrink-0">
          <BaseButton v-if="selectedOrder.status === 'Cooking'" variant="secondary" @click="emit('update-status', 'Cooked')" class="w-full sm:flex-1 py-3 sm:py-2">
            Mark Cooked
          </BaseButton>
          <BaseButton variant="success" @click="emit('settle-payment')" :class="[selectedOrder.status !== 'Cooking' ? 'col-span-2' : '', 'w-full sm:flex-1 py-3 sm:py-2']">
            Settle Payment
          </BaseButton>
        </div>
      </div>

    </div>
  </div>
</template>