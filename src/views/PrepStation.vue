<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { inventoryService, type RawInventoryItem, type PreparedInventoryItem } from '../services/inventoryService';
import { staffService } from '../services/staffService';
import Header from '../components/layout/Header.vue';
import DataLoader from '../components/ui/DataLoader.vue';
import BaseButton from '../components/ui/BaseButton.vue';
import { useResponsive } from '../composables/useResponsive';

const { fontSm, fontBase } = useResponsive();

const selectedCategory = ref('');
const selectedPart = ref('');
const selectedPrepItemId = ref<number | "">("");
const rawKilos = ref<number | null>(null);
const skewersProduced = ref<number | null>(null);
const selectedStaff = ref('');

const staffMembers = ref<any[]>([]);
const availableCategories = ref<string[]>([]);
const availableParts = ref<RawInventoryItem[]>([]);
const allPreparedItems = ref<PreparedInventoryItem[]>([]);
const currentStockInfo = ref<RawInventoryItem | null>(null);

const isLoadingData = ref(true);
const isLoadingParts = ref(false);

const maxKilosAllowed = computed(() => {
  if (!currentStockInfo.value) return 0;
  return currentStockInfo.value.current_stock_kg;
});

// FIX: Filters by matching Category instead of strictly relying on raw_item_id
const availableOutputItems = computed(() => {
  if (!currentStockInfo.value) return [];
  
  const matched = allPreparedItems.value.filter(item => 
    item.category === currentStockInfo.value!.category || 
    item.raw_item_id === currentStockInfo.value!.raw_item_id
  );

  // Fallback: If categories mismatch, show all items so the staff isn't blocked
  return matched.length > 0 ? matched : allPreparedItems.value;
});

const canPrep = computed(() => {
  if (!selectedCategory.value || !selectedPart.value || selectedPrepItemId.value === "" || !rawKilos.value || !skewersProduced.value || !selectedStaff.value) {
    return false;
  }
  if (!currentStockInfo.value) return false;
  return rawKilos.value > 0 && rawKilos.value <= currentStockInfo.value.current_stock_kg && skewersProduced.value > 0;
});

const stockStatus = computed(() => {
  if (!currentStockInfo.value) return { text: 'No stock info', class: 'text-on-surface-variant' };
  const stock = currentStockInfo.value.current_stock_kg;
  if (stock === 0) return { text: 'Out of stock', class: 'text-error' };
  if (stock < 1) return { text: 'Critically low', class: 'text-tertiary-container' };
  return { text: `${stock.toFixed(2)} kg available`, class: 'text-primary' };
});

async function loadStaff() {
  const result = await staffService.getAllStaff();
  staffMembers.value = (result as any[]).filter(staff => staff.status === 'Active');
  if (staffMembers.value.length > 0) {
    selectedStaff.value = staffMembers.value[0].full_name;
  }
}

async function loadCategories() {
  try {
    availableCategories.value = await inventoryService.getAvailableCategories();
    if (availableCategories.value.length > 0) {
      selectedCategory.value = availableCategories.value[0];
    }
  } catch (error) {
    console.error("Failed to load categories:", error);
  }
}

async function loadParts(category: string) {
  if (!category) {
    availableParts.value = [];
    selectedPart.value = '';
    return;
  }
  
  isLoadingParts.value = true;
  try {
    availableParts.value = await inventoryService.getAvailableParts(category);
    if (availableParts.value.length > 0) {
      selectedPart.value = availableParts.value[0].specific_part;
    } else {
      selectedPart.value = '';
    }
  } catch (error) {
    console.error("Failed to load parts:", error);
  } finally {
    isLoadingParts.value = false;
  }
}

function updateCurrentStockInfo() {
  if (selectedCategory.value && selectedPart.value) {
    currentStockInfo.value = availableParts.value.find(
      p => p.specific_part === selectedPart.value
    ) || null;
  } else {
    currentStockInfo.value = null;
  }
  // Reset selected output when the meat changes
  selectedPrepItemId.value = "";
}

async function handleSavePrep() {
  if (!canPrep.value) {
    if (rawKilos.value && currentStockInfo.value && rawKilos.value > currentStockInfo.value.current_stock_kg) {
      alert(`Insufficient stock! Available: ${currentStockInfo.value.current_stock_kg.toFixed(2)}kg, Requested: ${rawKilos.value}kg`);
    } else {
      alert("Please fill out all fields correctly.");
    }
    return;
  }

  try {
    await inventoryService.logPrepTransaction(
      selectedCategory.value, 
      selectedPart.value, 
      rawKilos.value!, 
      skewersProduced.value!,
      Number(selectedPrepItemId.value),
      selectedStaff.value
    );

    await loadCategories();
    await loadParts(selectedCategory.value);
    
    // Refresh all prepared items so we have the latest variant counts
    allPreparedItems.value = await inventoryService.getPreparedInventory();
    
    rawKilos.value = null;
    skewersProduced.value = null;
    selectedPrepItemId.value = "";
    
    alert("Prep log successfully saved! Inventory has been updated.");

  } catch (error) {
    console.error("Failed to log prep:", error);
    alert(`Error: ${error instanceof Error ? error.message : 'Database error occurred'}`);
  }
}

watch(selectedCategory, async (newCategory) => {
  if (newCategory) {
    await loadParts(newCategory);
  }
});

watch([selectedPart, availableParts], () => {
  updateCurrentStockInfo();
});

onMounted(async () => {
  isLoadingData.value = true;
  try {
    await loadStaff();
    await loadCategories();
    allPreparedItems.value = await inventoryService.getPreparedInventory();
  } finally {
    setTimeout(() => {
      isLoadingData.value = false;
    }, 600);
  }
});
</script>

<template>
  <div class="w-full max-w-400 mx-auto h-full flex flex-col">
    
    <div v-if="isLoadingData" class="w-full h-full bg-surface-container-low rounded-2xl shadow-sm border border-outline-variant/15 flex items-center justify-center">
      <DataLoader message="Loading preparation environment..." />
    </div>

    <div v-else class="w-full h-full bg-surface-container-low border border-outline-variant/15 rounded-2xl shadow-sm flex flex-col overflow-hidden">
      
      <Header 
        title="Prep Station" 
        subtitle="Log skewering tasks"
        customClass="p-4 md:px-6 md:py-4 border-b border-outline-variant/10 bg-surface-container-highest/30"
      />

      <div class="flex-1 overflow-y-auto p-4 md:p-6 bg-surface">
        <form @submit.prevent="handleSavePrep" class="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-5xl mx-auto">
          
          <div class="col-span-1">
            <label :class="['block font-bold text-on-surface-variant uppercase tracking-widest mb-2', fontSm]">Meat Category</label>
            <select 
              v-model="selectedCategory" 
              :disabled="availableCategories.length === 0"
              :class="['w-full bg-surface-container text-on-surface border border-outline-variant/30 rounded-xl px-4 py-3 font-medium focus:border-primary-container focus:ring-1 focus:ring-primary-container outline-none disabled:opacity-50 disabled:cursor-not-allowed transition-colors', fontBase]"
            >
              <option value="">Select Category</option>
              <option v-for="cat in availableCategories" :key="cat" :value="cat">{{ cat }}</option>
            </select>
            <p v-if="availableCategories.length === 0" :class="['text-error mt-2 font-bold uppercase tracking-widest text-[10px]', fontSm]">
              No categories with available stock
            </p>
          </div>
          
          <div class="col-span-1">
            <label :class="['block font-bold text-on-surface-variant uppercase tracking-widest mb-2', fontSm]">Specific Part / Cut</label>
            <select 
              v-model="selectedPart" 
              :disabled="!selectedCategory || isLoadingParts || availableParts.length === 0"
              :class="['w-full bg-surface-container text-on-surface border border-outline-variant/30 rounded-xl px-4 py-3 font-medium focus:border-primary-container focus:ring-1 focus:ring-primary-container outline-none disabled:opacity-50 disabled:cursor-not-allowed transition-colors', fontBase]"
            >
              <option value="">Select Part</option>
              <option v-for="part in availableParts" :key="part.raw_item_id" :value="part.specific_part">
                {{ part.specific_part }} ({{ part.current_stock_kg.toFixed(2) }} kg available)
              </option>
            </select>
            <p v-if="isLoadingParts" :class="['text-on-surface-variant font-bold uppercase tracking-widest mt-2 text-[10px]', fontSm]">Loading parts...</p>
          </div>
          
          <div v-if="currentStockInfo" class="md:col-span-2 bg-surface-container border border-outline-variant/20 p-4 rounded-xl flex items-center justify-between shadow-inner">
            <div>
              <p :class="['font-black text-lg', stockStatus.class, fontBase]">{{ stockStatus.text }}</p>
              <p :class="['text-on-surface-variant font-bold uppercase tracking-widest mt-1 text-[10px]', fontSm]">Alert threshold: {{ currentStockInfo.alert_threshold_kg }} kg</p>
            </div>
            <div class="hidden md:block text-on-surface-variant opacity-30">
              <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
            </div>
          </div>

          <div class="md:col-span-2 bg-primary-container/10 border border-primary-container/20 p-4 rounded-xl shadow-sm">
            <label :class="['block font-bold text-primary-container uppercase tracking-widest mb-2', fontSm]">Target Output Variant</label>
            <select 
              v-model="selectedPrepItemId" 
              :disabled="availableOutputItems.length === 0"
              required
              :class="['w-full bg-surface text-on-surface border border-outline-variant/30 rounded-xl px-4 py-3 font-medium focus:border-primary-container focus:ring-1 focus:ring-primary-container outline-none disabled:opacity-50 disabled:cursor-not-allowed transition-colors', fontBase]"
            >
              <option value="" disabled>-- Select the precise menu item being skewered --</option>
              <option v-for="item in availableOutputItems" :key="item.prep_item_id" :value="item.prep_item_id">
                {{ item.variant_group ? `${item.variant_group} - ${item.variant_name}` : item.pos_display_name }}
              </option>
            </select>
          </div>
          
          <div class="col-span-1">
            <label :class="['block font-bold text-on-surface-variant uppercase tracking-widest mb-2', fontSm]">
              Raw Kilos Used 
              <span v-if="maxKilosAllowed > 0" class="text-on-surface-variant/50 font-black ml-1">
                (Max: {{ maxKilosAllowed.toFixed(2) }} kg)
              </span>
            </label>
            <input 
              v-model.number="rawKilos" 
              type="number" 
              step="0.1"
              min="0.1"
              :max="maxKilosAllowed"
              :disabled="!currentStockInfo || currentStockInfo.current_stock_kg === 0"
              required 
              placeholder="e.g., 5.0" 
              :class="['w-full bg-surface-container text-on-surface placeholder-on-surface-variant/50 border border-outline-variant/30 rounded-xl px-4 py-3 font-bold focus:border-primary-container focus:ring-1 focus:ring-primary-container outline-none disabled:opacity-50 disabled:cursor-not-allowed transition-colors', fontBase]"
            />
          </div>
          
          <div class="col-span-1">
            <label :class="['block font-bold text-on-surface-variant uppercase tracking-widest mb-2', fontSm]">Skewers Produced</label>
            <input 
              v-model.number="skewersProduced" 
              type="number"
              min="1"
              :disabled="!currentStockInfo || currentStockInfo.current_stock_kg === 0"
              required 
              placeholder="e.g., 120" 
              :class="['w-full bg-surface-container text-on-surface placeholder-on-surface-variant/50 border border-outline-variant/30 rounded-xl px-4 py-3 font-bold focus:border-primary-container focus:ring-1 focus:ring-primary-container outline-none disabled:opacity-50 disabled:cursor-not-allowed transition-colors', fontBase]"
            />
          </div>
          
          <div class="col-span-1">
            <label :class="['block font-bold text-on-surface-variant uppercase tracking-widest mb-2', fontSm]">Staff Member</label>
            <select 
              v-model="selectedStaff" 
              required 
              :class="['w-full bg-surface-container text-on-surface border border-outline-variant/30 rounded-xl px-4 py-3 font-medium focus:border-primary-container focus:ring-1 focus:ring-primary-container outline-none transition-colors', fontBase]"
            >
              <option v-for="staff in staffMembers" :key="staff.staff_id" :value="staff.full_name">
                {{ staff.full_name }} ({{ staff.role }})
              </option>
            </select>
          </div>
          
          <div class="col-span-1 flex items-end mt-2 md:mt-0">
            <BaseButton 
              type="submit" 
              variant="primary"
              :disabled="!canPrep"
              :class="['w-full py-4 text-lg h-12.5 md:h-13.5', fontBase]"
            >
              <span class="font-black uppercase tracking-widest text-sm">Save Prep Log</span>
            </BaseButton>
          </div>
        </form>
      </div>
    </div>

  </div>
</template>