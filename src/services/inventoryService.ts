import { getApiBaseUrl } from '../utils/imageUtils'; // NEW: Import the centralized base URL

export interface RawInventoryItem {
  raw_item_id: number;
  category: string;
  specific_part: string;
  current_stock_kg: number;
  alert_threshold_kg: number;
}

export interface PreparedInventoryItem {
  prep_item_id: number;
  raw_item_id: number | null;
  category: string;
  pos_display_name: string;
  variant_group: string | null;
  variant_name: string | null; 
  current_stock_pieces: number;
  unit_price: number;
  photo_url?: string | null;
}

export interface POSCategory {
  category_name: string;
  is_removable: boolean;
}

export interface PrepLog {
  timestamp: string;
  staff_name: string;
  category: string;
  specific_part: string;
  kilos_deducted: number;
  skewers_added: number;
}

// NEW: Use the utility function instead of hardcoding the localhost logic here
const API_BASE = `${getApiBaseUrl()}/api`;
const CURRENT_ADMIN_ID = 1;

export const inventoryService = {
  async getRawInventory(): Promise<RawInventoryItem[]> {
    const res = await fetch(`${API_BASE}/inventory/raw`);
    if (!res.ok) throw new Error('Failed to fetch raw inventory');
    return await res.json();
  },
  
  async getPreparedInventory(): Promise<PreparedInventoryItem[]> {
    const res = await fetch(`${API_BASE}/inventory/prepared`);
    if (!res.ok) throw new Error('Failed to fetch prepared inventory');
    return await res.json();
  },

  async getPosCategories(): Promise<POSCategory[]> {
    const res = await fetch(`${API_BASE}/inventory/pos-categories`);
    if (!res.ok) throw new Error('Failed to fetch categories');
    return await res.json();
  },

  async addPosCategory(categoryName: string): Promise<void> {
    const res = await fetch(`${API_BASE}/inventory/pos-categories/add`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ category_name: categoryName })
    });
    if (!res.ok) throw new Error('Failed to add category');
  },

  async removePosCategory(categoryName: string): Promise<void> {
    const res = await fetch(`${API_BASE}/inventory/pos-categories/remove`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ category_name: categoryName })
    });
    if (!res.ok) throw new Error('Failed to remove category');
  },

  async editStock(itemType: 'raw' | 'prepared', itemId: number, quantityChange: number, reason: string): Promise<void> {
    const res = await fetch(`${API_BASE}/inventory/edit-stock`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ item_type: itemType, item_id: itemId, quantity_change: quantityChange, reason, staff_id: CURRENT_ADMIN_ID })
    });
    if (!res.ok) throw new Error('Failed to edit stock');
  },
  
  async addNewRawItem(category: string, part: string, initialKilos: number, alertThreshold: number): Promise<void> {
    const res = await fetch(`${API_BASE}/inventory/add-raw`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ category, part, initial_kilos: initialKilos, alert_threshold: alertThreshold, staff_id: CURRENT_ADMIN_ID })
    });
    if (!res.ok) throw new Error('Failed to add new raw item');
  },

  async addPreparedItem(category: string, posDisplayName: string, unitPrice: number, photoUrl?: string | null, variantGroup?: string | null, variantName?: string | null): Promise<void> {
    const res = await fetch(`${API_BASE}/inventory/add-prepared`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        category, 
        pos_display_name: posDisplayName, 
        variant_group: variantGroup || null,
        variant_name: variantName || null,
        unit_price: unitPrice, 
        photo_url: photoUrl || null,
        staff_id: CURRENT_ADMIN_ID 
      })
    });
    if (!res.ok) throw new Error('Failed to add prepared item');
  },
  
  async updatePreparedItemPricing(prepItemId: number, newPrice: number, photoUrl: string | null, variantGroup?: string | null, variantName?: string | null): Promise<void> {
    const res = await fetch(`${API_BASE}/inventory/update-pricing`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ 
        prep_item_id: prepItemId, 
        new_price: newPrice, 
        variant_group: variantGroup || null,
        variant_name: variantName || null,
        photo_url: photoUrl,
        staff_id: CURRENT_ADMIN_ID 
      })
    });
    if (!res.ok) throw new Error('Failed to update pricing');
  },
  
  async getAvailableCategories(): Promise<string[]> {
    const res = await fetch(`${API_BASE}/inventory/raw-categories`);
    if (!res.ok) throw new Error('Failed to fetch categories');
    return await res.json();
  },
  
  async getAvailableParts(category: string): Promise<RawInventoryItem[]> {
    const res = await fetch(`${API_BASE}/inventory/parts?category=${encodeURIComponent(category)}`);
    if (!res.ok) throw new Error('Failed to fetch parts');
    return await res.json();
  },
  
  async logPrepTransaction(category: string, part: string, kilos: number, sticks: number, prepItemId: number, staffName?: string): Promise<void> {
    const res = await fetch(`${API_BASE}/inventory/log-prep`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ category, part, kilos, sticks, prep_item_id: prepItemId, staff_name: staffName ?? null })
    });
    if (!res.ok) throw new Error('Failed to log prep transaction');
  },
  
  async getLowStockRawItems(): Promise<RawInventoryItem[]> {
    const res = await fetch(`${API_BASE}/dashboard/low-stock`);
    if (!res.ok) throw new Error('Failed to fetch low stock alerts');
    return await res.json();
  },
  
  async getRecentPrepLogs(limit: number = 10): Promise<PrepLog[]> {
    const res = await fetch(`${API_BASE}/inventory/recent-prep?limit=${limit}`);
    if (!res.ok) throw new Error('Failed to fetch recent prep logs');
    return await res.json();
  },

  async deletePreparedItem(prepItemId: number): Promise<void> {
    const res = await fetch(`${API_BASE}/inventory/delete-prepared`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prep_item_id: prepItemId })
    });
    if (!res.ok) throw new Error('Failed to delete item');
  },
};