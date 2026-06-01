<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { posService, type ActiveOrder, type PosItem } from '../services/posService';
import { queueService, type PendingOrder } from '../services/queueService';
import Header from '../components/layout/Header.vue';
import CashierOrderDetails from '../components/ui/CashierOrderDetails.vue';
import ActiveTabsModal from '../components/modal/ActiveTabsModal.vue';
import ReceiptModal from '../components/modal/ReceiptModal.vue';
import PrintConfirmationModal from '../components/modal/PrintConfirmationModal.vue';
import BaseButton from '../components/ui/BaseButton.vue';
import { useResponsive } from '../composables/useResponsive';
import { useAuth } from '../stores/authStore';

const { fontSm, fontBase, fontLg, isMobile, isTablet } = useResponsive();
const { currentUser } = useAuth();

const activeOrders = ref<ActiveOrder[]>([]);
const selectedOrder = ref<ActiveOrder | null>(null);
const availableItems = ref<PosItem[]>([]);

const selectedPending = ref<PendingOrder | null>(null);
const searchQueueInput = ref('');
const isSearching = ref(false);

const tableNumberInput = ref('');
const isLoadingData = ref(true);
const showTabsModal = ref(false);
const isReceiptModalOpen = ref(false);

// Post-payment state
const isPrintConfirmOpen = ref(false);
const justSettledOrder = ref<ActiveOrder | null>(null);

const staffId = currentUser.value?.id || 1;

async function loadData() {
  isLoadingData.value = true;
  try {
    activeOrders.value = await posService.getActiveOrders();
    availableItems.value = await posService.getAvailablePosItems(); 
    
    if (selectedOrder.value) {
      selectedOrder.value = activeOrders.value.find(o => o.order_id === selectedOrder.value?.order_id) || null;
    }
  } catch (error) {
    console.error("Failed to load tabs:", error);
  } finally {
    setTimeout(() => { isLoadingData.value = false; }, 600);
  }
}

function selectOrder(order: ActiveOrder) {
  selectedOrder.value = order;
  selectedPending.value = null; 
}

async function handleSearchQueue() {
  if (!searchQueueInput.value) return;
  
  isSearching.value = true;
  try {
    const queueNum = parseInt(searchQueueInput.value);
    const currentQueue = await queueService.getQueue();
    const found = currentQueue.find(q => q.queue_number === queueNum);
    
    if (found) {
      found.cart_items = found.cart_items.map((item: any) => ({
        prep_item_id: item.prep_item_id,
        pos_display_name: item.pos_display_name,
        qty: item.qty,
        unit_price: item.unit_price,
        is_variable_price: item.is_variable_price || false,
        photo_url: item.photo_url || null,
        category: "Kiosk Order",
        current_stock_pieces: 0,
        variant_group: null,
        variant_name: null
      }));

      selectedPending.value = found;
      selectedOrder.value = null; 
      searchQueueInput.value = ''; 
      
      if (found.order_type === 'Dine-in') {
        const nextNum = await posService.getNextTableNumber();
        tableNumberInput.value = nextNum.toString();
      } else {
        tableNumberInput.value = ''; 
      }
      
    } else {
      alert(`Queue #${queueNum} not found. It may have already been processed.`);
    }
  } catch (error) {
    console.error("Search failed:", error);
    alert("Error communicating with the queue system.");
  } finally {
    isSearching.value = false;
  }
}

async function handleUpdateStatus(newStatus: string) {
  if (!selectedOrder.value) return;
  try {
    await posService.updateOrderStatus(selectedOrder.value.order_id, newStatus, staffId);
    await loadData();
  } catch (error) {
    console.error("Failed to update status:", error);
    alert("An error occurred while updating the order status.");
  }
}

async function handleSettlePayment() {
  if (!selectedOrder.value) return;
  const confirmPayment = confirm(`Settle payment of PHP ${selectedOrder.value.total_amount.toFixed(2)} for ${selectedOrder.value.customer_identifier}?`);
  if (!confirmPayment) return;

  try {
    await posService.settlePayment(selectedOrder.value.order_id, staffId);
    
    // Save order data for the confirmation dialog before clearing the active selection
    justSettledOrder.value = selectedOrder.value;
    isPrintConfirmOpen.value = true;
    
    selectedOrder.value = null;
    await loadData();
  } catch (error) {
    console.error("Failed to settle payment:", error);
    alert("An error occurred while settling payment.");
  }
}

async function handleAcceptPending() {
  if (!selectedPending.value) return;
  
  if (!tableNumberInput.value) {
    alert("Please enter the Table Number (1-100).");
    return;
  }

  try {
    const finalIdentifier = `Queue #${selectedPending.value.queue_number} - Table ${tableNumberInput.value}`;

    await posService.sendToGrill(
      staffId,
      finalIdentifier, 
      selectedPending.value.order_type,
      selectedPending.value.cart_items,
      selectedPending.value.subtotal,
      selectedPending.value.tax,
      selectedPending.value.total
    );

    await queueService.removeFromQueue(selectedPending.value.queue_number);
    
    alert(`${finalIdentifier} accepted and sent to grill!`);
    selectedPending.value = null;
    tableNumberInput.value = ''; 
    await loadData(); 
  } catch (error) {
    console.error("Failed to accept pending order:", error);
    alert("Failed to process queue order.");
  }
}

async function handleRejectPending() {
  if (!selectedPending.value) return;
  const confirmReject = confirm(`Are you sure you want to delete Queue #${selectedPending.value.queue_number}? This cannot be undone.`);
  if (!confirmReject) return;

  try {
    await queueService.removeFromQueue(selectedPending.value.queue_number);
    selectedPending.value = null;
    tableNumberInput.value = '';
  } catch(error) {
    console.error("Failed to reject pending order", error);
  }
}

async function handleSaveActiveEdit(updatedOrder: ActiveOrder) {
  try {
    await posService.editActiveOrder(
      updatedOrder.order_id, 
      staffId, 
      updatedOrder.cart_items as any, 
      updatedOrder.total_amount, 
      0, 
      updatedOrder.total_amount
    );
    await loadData();
    alert("Order successfully updated.");
  } catch (error) {
    console.error("Failed to edit order:", error);
    alert("An error occurred saving the edit.");
  }
}

function handleSavePendingEdit(updatedPending: PendingOrder) {
  selectedPending.value = updatedPending;
}

// Handlers for post-payment print confirmation
async function handlePostPaymentPrint() {
  if (!justSettledOrder.value) return;
  
  isPrintConfirmOpen.value = false;
  try {
    await posService.reprintReceipt(justSettledOrder.value.order_id);
  } catch (error) {
    console.error("Failed to print receipt:", error);
    alert("Failed to communicate with the printer.");
  }
}

function handlePostPaymentView() {
  isPrintConfirmOpen.value = false;
  isReceiptModalOpen.value = true;
}

// Handler for manual printing from the preview modal
// Inside Cashier.vue
async function handleManualPrint(orderId: number) {
  try {
    await posService.reprintReceipt(orderId);
    alert("Receipt successfully printed!");
  } catch (error) {
    console.error("Print job failed:", error);
    // This will now trigger if the printer is off, out of paper, or disconnected
    alert("PRINTER ERROR: Please check if the Xprinter is turned on and connected to the PC.");
  }
}

onMounted(() => {
  loadData();
});
</script>

<template>
  <div class="w-full max-w-400 mx-auto h-full flex flex-col gap-4 md:gap-6 items-stretch">
    
    <Header 
      title="Order Details" 
      subtitle="Manage and fulfill selected tab"
      customClass="bg-surface-container-low border border-outline-variant/15 rounded-2xl px-4 sm:px-5 py-3 sm:py-4 shadow-sm"
    >
      <template #actions>
        <div class="relative flex-1 sm:w-48 lg:w-56 min-w-[140px]">
          <input 
            v-model="searchQueueInput" 
            @keyup.enter="handleSearchQueue"
            type="number" 
            placeholder="Queue #" 
            :class="['w-full bg-surface-container text-on-surface placeholder-on-surface-variant/50 border border-outline-variant/30 rounded-full pl-9 pr-3 py-1.5 focus:outline-none focus:border-primary-container focus:ring-1 focus:ring-primary-container transition-colors', fontSm]"
          />
          <svg class="w-4 h-4 text-on-surface-variant absolute left-3 top-1/2 -translate-y-1/2" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
        </div>
        
        <BaseButton variant="secondary" @click="handleSearchQueue" :disabled="!searchQueueInput || isSearching" class="py-1.5 px-4 rounded-full!">
          <span :class="fontSm">{{ isSearching ? '...' : 'Search' }}</span>
        </BaseButton>

        <div class="hidden sm:block w-px h-6 bg-outline-variant/30 mx-1"></div>

        <button @click="showTabsModal = true" class="relative bg-surface-container-high hover:bg-surface-variant border border-outline-variant/20 px-4 py-1.5 rounded-full transition-colors flex items-center gap-2">
          <svg class="w-5 h-5 text-on-surface" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"></path></svg>
          <span :class="['font-bold text-on-surface hidden sm:block', fontSm]">Active Tabs</span>
          <span v-if="activeOrders.length > 0" class="absolute -top-1.5 -right-1.5 bg-error text-on-error text-[10px] font-black px-1.5 py-0.5 rounded-full shadow-sm">
            {{ activeOrders.length }}
          </span>
        </button>
      </template>
    </Header>

    <div class="w-full h-full min-h-0 flex-1 transition-all duration-300">
      <CashierOrderDetails 
        :selected-order="selectedOrder"
        :selected-pending="selectedPending"
        :available-items="availableItems"
        :table-number-input="tableNumberInput"
        @update:table-number-input="tableNumberInput = $event"
        @clear-pending="selectedPending = null"
        @reject-pending="handleRejectPending"
        @accept-pending="handleAcceptPending"
        @update-status="handleUpdateStatus"
        @settle-payment="handleSettlePayment"
        @save-active-edit="handleSaveActiveEdit"
        @save-pending-edit="handleSavePendingEdit"
        @preview-receipt="isReceiptModalOpen = true" 
      />
    </div>

    <ActiveTabsModal
      :show="showTabsModal"
      :active-orders="activeOrders"
      :is-loading-data="isLoadingData"
      :selected-order-id="selectedOrder?.order_id"
      @close="showTabsModal = false"
      @select-order="selectOrder"
    />

    <ReceiptModal 
      :is-open="isReceiptModalOpen" 
      :order="selectedOrder || justSettledOrder" 
      @close="isReceiptModalOpen = false"
      @print="handleManualPrint"
    />

    <PrintConfirmationModal
      :is-open="isPrintConfirmOpen"
      :customer-identifier="justSettledOrder?.customer_identifier || ''"
      @print="handlePostPaymentPrint"
      @view="handlePostPaymentView"
      @close="isPrintConfirmOpen = false"
    />

  </div>
</template>