<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { getImageUrl } from '../utils/imageUtils';
import type { RawInventoryItem, PreparedInventoryItem, POSCategory } from '../services/inventoryService';
import { inventoryService } from '../services/inventoryService';
import Header from '../components/layout/Header.vue';
import EditInventoryModal from '../components/modal/EditInventoryModal.vue';
import EditPriceModal from '../components/modal/EditPriceModal.vue';
import AddVariantModal from '../components/modal/AddVariantModal.vue';
import DataLoader from '../components/ui/DataLoader.vue';
import BaseButton from '../components/ui/BaseButton.vue';
import BaseBadge from '../components/ui/BaseBadge.vue';
import { useResponsive } from '../composables/useResponsive';

const { fontSm, fontBase, fontXl, isMobile } = useResponsive();

const rawInventory = ref<RawInventoryItem[]>([]);
const preparedInventory = ref<PreparedInventoryItem[]>([]);
const posCategories = ref<POSCategory[]>([]);
const viewMode = ref('raw'); 

const isLoadingData = ref(true);

const isInventoryModalOpen = ref(false);
const isPriceModalOpen = ref(false);
const isAddVariantModalOpen = ref(false);

const selectedPreparedItem = ref<PreparedInventoryItem | null>(null);
const selectedGroupForVariant = ref<any | null>(null);

const expandedGroups = ref<Set<string>>(new Set());

async function loadData() {
  isLoadingData.value = true;
  try {
    rawInventory.value = await inventoryService.getRawInventory();
    preparedInventory.value = await inventoryService.getPreparedInventory();
    posCategories.value = await inventoryService.getPosCategories();
  } catch (error) {
    console.error("Error loading inventory:", error);
  } finally {
    setTimeout(() => { isLoadingData.value = false; }, 600);
  }
}

const groupedPreparedItems = computed(() => {
  const items = preparedInventory.value.filter(item => item.category === viewMode.value);
  const groups: any[] = [];
  const groupMap = new Map();

  items.forEach(item => {
    if (item.variant_group) {
      if (groupMap.has(item.variant_group)) {
        const group = groupMap.get(item.variant_group);
        group.variants.push(item);
        group.total_stock += item.current_stock_pieces;
      } else {
        const newGroup = {
          is_group: true,
          group_name: item.variant_group,
          total_stock: item.current_stock_pieces,
          photo_url: item.photo_url,
          variants: [item]
        };
        groupMap.set(item.variant_group, newGroup);
        groups.push(newGroup);
      }
    } else {
      groups.push({
        is_group: false,
        item: item
      });
    }
  });
  return groups;
});

function toggleGroup(groupName: string) {
  if (expandedGroups.value.has(groupName)) {
    expandedGroups.value.delete(groupName);
  } else {
    expandedGroups.value.add(groupName);
  }
}

function openEditPriceModal(item: PreparedInventoryItem) {
  selectedPreparedItem.value = item;
  isPriceModalOpen.value = true;
}

function openAddVariantModal(groupOrItem: any) {
  selectedGroupForVariant.value = groupOrItem;
  isAddVariantModalOpen.value = true;
}

async function handleSavePrice(data: { prepItemId: number; unitPrice: number; photoUrl: string | null; variantGroup: string | null; variantName: string | null }) {
  try {
    await inventoryService.updatePreparedItemPricing(data.prepItemId, data.unitPrice, data.photoUrl, data.variantGroup, data.variantName);
    isPriceModalOpen.value = false;
    await loadData();
  } catch (error) {
    alert("Failed to update item details.");
  }
}

async function handleSaveNewVariant(data: { name: string; price: number; photoUrl: string | null }) {
  try {
    if (!selectedGroupForVariant.value.is_group && !selectedGroupForVariant.value.item.variant_group) {
      const baseItem = selectedGroupForVariant.value.item;
      await inventoryService.updatePreparedItemPricing(baseItem.prep_item_id, baseItem.unit_price, baseItem.photo_url, baseItem.pos_display_name, "Regular");
      await inventoryService.addPreparedItem(baseItem.category, baseItem.pos_display_name, data.price, data.photoUrl, baseItem.pos_display_name, data.name);
    } else {
      const groupName = selectedGroupForVariant.value.is_group ? selectedGroupForVariant.value.group_name : selectedGroupForVariant.value.item.variant_group;
      const category = selectedGroupForVariant.value.is_group ? selectedGroupForVariant.value.variants[0].category : selectedGroupForVariant.value.item.category;
      const displayName = selectedGroupForVariant.value.is_group ? selectedGroupForVariant.value.variants[0].pos_display_name : selectedGroupForVariant.value.item.pos_display_name;

      await inventoryService.addPreparedItem(category, displayName, data.price, data.photoUrl, groupName, data.name);
    }
    isAddVariantModalOpen.value = false;
    await loadData();
  } catch (error) {
    alert("Failed to add variant.");
  }
}

async function handleDeleteItem(prepItemId: number, name: string) {
  if (!confirm(`Are you sure you want to remove ${name}?`)) return;
  try {
    await inventoryService.deletePreparedItem(prepItemId);
    await loadData();
  } catch (error: any) {
    alert(error.message || "Failed to delete item.");
  }
}

async function handleDeleteGroup(group: any) {
  if (!confirm(`Are you sure you want to remove the entire ${group.group_name} group and all its variants?`)) return;
  try {
    for (const variant of group.variants) {
      await inventoryService.deletePreparedItem(variant.prep_item_id);
    }
    await loadData();
  } catch (error: any) {
    alert("Failed to delete some items. They may have active sales records.");
  }
}

function getStatusBadge(current: number, threshold: number): 'danger' | 'warning' | 'success' {
  if (current <= threshold * 0.5) return 'danger';
  if (current <= threshold) return 'warning';
  return 'success';
}

function getStatusText(current: number, threshold: number): string {
  if (current <= threshold * 0.5) return 'Critically Low';
  if (current <= threshold) return 'Low Stock';
  return 'Adequate';
}

function handleImageError(event: Event) {
  const target = event.target as HTMLImageElement;
  if (target) target.style.display = 'none';
}

onMounted(() => loadData());
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden">
    <div class="bg-surface-container-low p-4 md:p-6 rounded-2xl shadow-sm border border-outline-variant/15 flex-1 flex flex-col min-h-0 overflow-hidden">
      
      <Header 
        title="Inventory Management" 
        :subtitle="viewMode === 'raw' ? 'Monitored in Kilograms (kg)' : 'Monitored in Sticks/Pieces'"
        customClass="mb-4"
      >
        <template #actions>
          <BaseButton variant="primary" @click="isInventoryModalOpen = true" :class="[isMobile ? 'w-full' : '', 'py-2 px-4 text-xs rounded-lg h-9.5 shadow-sm hover:shadow transition-shadow']">
            <svg class="w-4 h-4 mr-1.5 inline" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg>
            <span class="tracking-widest uppercase font-bold">Add Item</span>
          </BaseButton>
        </template>
      </Header>

      <div class="flex overflow-x-auto pb-4 mb-2 gap-2 border-b border-outline-variant/10 shrink-0">
        <button @click="viewMode = 'raw'" :class="[viewMode === 'raw' ? 'bg-primary-container text-on-primary-container shadow-sm border-primary-container' : 'bg-surface-container text-on-surface-variant hover:bg-surface-container-high border-transparent', 'px-6 py-2.5 font-bold uppercase tracking-widest text-xs rounded-lg transition-colors shrink-0 border']">
          Raw Meats
        </button>
        <button v-for="cat in posCategories" :key="cat.category_name" @click="viewMode = cat.category_name" :class="[viewMode === cat.category_name ? 'bg-primary-container text-on-primary-container shadow-sm border-primary-container' : 'bg-surface-container text-on-surface-variant hover:bg-surface-container-high border-transparent', 'px-6 py-2.5 font-bold uppercase tracking-widest text-xs rounded-lg transition-colors shrink-0 border']">
          {{ cat.category_name }}
        </button>
      </div>

      <DataLoader v-if="isLoadingData" message="Fetching latest inventory records..." />

      <div v-else class="flex-1 overflow-auto bg-surface border border-outline-variant/15 rounded-xl">
        <div v-if="viewMode === 'raw'">
          <table class="w-full text-left border-collapse">
            <thead class="bg-surface-container sticky top-0 z-10 shadow-sm outline outline-outline-variant/10">
              <tr :class="['text-on-surface-variant uppercase tracking-widest', fontSm]">
                <th class="py-4 font-bold px-4">Category</th>
                <th class="py-4 font-bold px-4">Specific Part</th>
                <th class="py-4 font-bold px-4">Current Stock</th>
                <th class="py-4 font-bold px-4 hidden md:table-cell">Threshold</th>
                <th class="py-4 font-bold px-4 text-right">Status</th>
              </tr>
            </thead>
            <tbody class="text-on-surface divide-y divide-outline-variant/10">
              <tr v-if="rawInventory.length === 0">
                <td colspan="5" class="py-12 text-center text-on-surface-variant font-bold uppercase tracking-widest text-sm">No raw inventory found.</td>
              </tr>
              <tr v-for="item in rawInventory" :key="item.raw_item_id" class="hover:bg-surface-container-high transition-colors">
                <td :class="['py-4 font-black text-on-surface px-4', fontBase]">{{ item.category }}</td>
                <td :class="['py-4 font-bold text-on-surface-variant px-4', fontBase]">{{ item.specific_part }}</td>
                <td :class="['py-4 font-black px-4', item.current_stock_kg <= item.alert_threshold_kg ? 'text-error' : 'text-primary', fontBase]">
                  {{ item.current_stock_kg.toFixed(2) }} kg
                </td>
                <td :class="['py-4 text-on-surface-variant font-bold px-4 hidden md:table-cell', fontBase]">{{ item.alert_threshold_kg }} kg</td>
                <td class="py-4 px-4 text-right">
                  <BaseBadge :text="getStatusText(item.current_stock_kg, item.alert_threshold_kg)" :variant="getStatusBadge(item.current_stock_kg, item.alert_threshold_kg)" />
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div v-else>
          <table class="w-full text-left border-collapse">
            <thead class="bg-surface-container sticky top-0 z-10 shadow-sm outline outline-outline-variant/10">
              <tr :class="['text-on-surface-variant uppercase tracking-widest', fontSm]">
                <th class="py-4 font-bold px-4 w-1/3">Item Name</th>
                <th class="py-4 font-bold px-4">Stock Count</th>
                <th class="py-4 font-bold px-4">Price (PHP)</th>
                <th class="py-4 font-bold px-4 text-right">Actions</th>
              </tr>
            </thead>
            <tbody class="text-on-surface divide-y divide-outline-variant/10">
              <tr v-if="groupedPreparedItems.length === 0">
                <td colspan="4" class="py-12 text-center text-on-surface-variant font-bold uppercase tracking-widest text-sm">No items found in this category.</td>
              </tr>
              
              <template v-for="(group, index) in groupedPreparedItems" :key="index">
                <tr v-if="!group.is_group" class="hover:bg-surface-container-high transition-colors group">
                  <td :class="['py-3 px-4', fontBase]">
                    <div class="flex items-center gap-4">
                      <div v-if="group.item.photo_url" class="w-12 h-12 rounded-lg bg-surface-container flex items-center justify-center overflow-hidden shrink-0 border border-outline-variant/20 shadow-sm">
                        <img :src="getImageUrl(group.item.photo_url)" class="w-full h-full object-cover" @error="handleImageError" />
                      </div>
                      <div v-else class="w-12 h-12 rounded-lg bg-surface-container flex items-center justify-center shrink-0 border border-outline-variant/10 text-on-surface-variant opacity-50">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path></svg>
                      </div>
                      <span class="font-black text-on-surface">{{ group.item.pos_display_name }}</span>
                    </div>
                  </td>
                  <td :class="['py-3 font-black text-on-surface px-4', fontBase]">{{ group.item.current_stock_pieces }}</td>
                  <td :class="['py-3 font-black text-primary px-4', fontBase]">₱{{ group.item.unit_price.toFixed(2) }}</td>
                  <td class="py-3 px-4 text-right">
                    <div class="flex justify-end gap-3 items-center opacity-0 group-hover:opacity-100 transition-opacity">
                      <button @click="openAddVariantModal(group)" class="text-success hover:text-success/80 font-bold px-2 py-1 text-xs uppercase tracking-widest flex items-center gap-1"><svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg> Add Variant</button>
                      <button @click="openEditPriceModal(group.item)" class="text-primary hover:text-primary-container font-bold px-2 py-1 text-xs uppercase tracking-widest">Edit</button>
                      <button @click="handleDeleteItem(group.item.prep_item_id, group.item.pos_display_name)" class="text-error hover:text-error/80 p-1"><svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path></svg></button>
                    </div>
                  </td>
                </tr>

                <tr v-else class="hover:bg-surface-container-high transition-colors group cursor-pointer" @click="toggleGroup(group.group_name)">
                  <td :class="['py-3 px-4', fontBase]">
                    <div class="flex items-center gap-4">
                      <div v-if="group.photo_url" class="w-12 h-12 rounded-lg bg-surface-container flex items-center justify-center overflow-hidden shrink-0 border border-outline-variant/20 shadow-sm opacity-90">
                        <img :src="getImageUrl(group.photo_url)" class="w-full h-full object-cover" @error="handleImageError" />
                      </div>
                      <div v-else class="w-12 h-12 rounded-lg bg-surface-container flex items-center justify-center shrink-0 border border-outline-variant/10 text-on-surface-variant opacity-50">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path></svg>
                      </div>
                      <div class="flex flex-col">
                        <span class="font-black text-on-surface">{{ group.group_name }}</span>
                        <span class="text-[10px] text-primary font-bold mt-0.5 uppercase tracking-widest">{{ group.variants.length }} Variants</span>
                      </div>
                    </div>
                  </td>
                  <td :class="['py-3 font-black text-on-surface px-4', fontBase]">{{ group.total_stock }}</td>
                  <td :class="['py-3 font-bold text-on-surface-variant px-4', fontBase]">Multiple Prices</td>
                  <td class="py-3 px-4 text-right">
                    <div class="flex justify-end gap-3 items-center opacity-0 group-hover:opacity-100 transition-opacity">
                      <button @click.stop="openAddVariantModal(group)" class="text-success hover:text-success/80 font-bold px-2 py-1 text-xs uppercase tracking-widest flex items-center gap-1"><svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"></path></svg> Add Variant</button>
                      <button @click.stop="handleDeleteGroup(group)" class="text-error hover:text-error/80 p-1"><svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path></svg></button>
                      <svg :class="['w-4 h-4 ml-2 text-on-surface-variant transition-transform', expandedGroups.has(group.group_name) ? 'rotate-180' : '']" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path></svg>
                    </div>
                  </td>
                </tr>

                <template v-if="group.is_group && expandedGroups.has(group.group_name)">
                  <tr v-for="variant in group.variants" :key="variant.prep_item_id" class="bg-surface-container-low hover:bg-surface-container transition-colors group/variant">
                    <td :class="['py-2 px-4 pl-20', fontBase]">
                      <div class="flex items-center gap-3">
                        <svg class="w-4 h-4 text-outline-variant" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path></svg>
                        <span class="font-bold text-on-surface-variant">{{ variant.variant_name || variant.pos_display_name }}</span>
                      </div>
                    </td>
                    <td :class="['py-2 font-bold text-on-surface-variant px-4', fontBase]">{{ variant.current_stock_pieces }}</td>
                    <td :class="['py-2 font-black text-primary px-4', fontBase]">₱{{ variant.unit_price.toFixed(2) }}</td>
                    <td class="py-2 px-4 text-right">
                      <div class="flex justify-end gap-3 items-center mr-6 opacity-0 group-hover/variant:opacity-100 transition-opacity">
                        <button @click="openEditPriceModal(variant)" :class="['text-primary hover:text-primary-container font-bold px-2 py-1 transition-colors uppercase tracking-widest text-[10px]']">Edit</button>
                        <button @click="handleDeleteItem(variant.prep_item_id, variant.variant_name || variant.pos_display_name)" class="text-error hover:text-error/80 p-1"><svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path></svg></button>
                      </div>
                    </td>
                  </tr>
                </template>
              </template>
              
            </tbody>
          </table>
        </div>
      </div> 
    </div>

    <EditInventoryModal :is-open="isInventoryModalOpen" @close="isInventoryModalOpen = false" @refresh="loadData" />
    <EditPriceModal :is-open="isPriceModalOpen" :item="selectedPreparedItem" @close="isPriceModalOpen = false; selectedPreparedItem = null" @save="handleSavePrice" />
    <AddVariantModal :is-open="isAddVariantModalOpen" :base-item-name="(selectedGroupForVariant?.is_group ? selectedGroupForVariant?.group_name : selectedGroupForVariant?.item?.pos_display_name) || null" @close="isAddVariantModalOpen = false; selectedGroupForVariant = null" @save="handleSaveNewVariant" />
  </div>
</template>