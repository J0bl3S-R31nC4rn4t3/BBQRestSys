const API_BASE = window.location.hostname === 'localhost' 
  ? 'http://localhost:3000/api' 
  : `http://${window.location.hostname}:3000/api`;

export interface PosItem {
  prep_item_id: number;
  category: string;
  pos_display_name: string;
  variant_group: string | null;
  variant_name: string | null;
  current_stock_pieces: number;
  unit_price: number;
  photo_url: string | null;
}

export interface CartItem extends PosItem {
  qty: number;
}

export interface ActiveOrderItem {
  prep_item_id: number;
  pos_display_name: string;
  qty: number;
  unit_price: number;
}

export interface ActiveOrder {
  order_id: number;
  customer_identifier: string;
  order_type: string;
  total_amount: number;
  status: string;
  timestamp: string;
  cart_items: ActiveOrderItem[];
}

export const posService = {
  async getAvailablePosItems(): Promise<PosItem[]> {
    const res = await fetch(`${API_BASE}/inventory/prepared`);
    if (!res.ok) throw new Error('Failed to fetch POS items');
    return await res.json();
  },

  async getActiveOrders(): Promise<ActiveOrder[]> {
    const res = await fetch(`${API_BASE}/pos/active-orders`);
    if (!res.ok) throw new Error('Failed to fetch active orders');
    return await res.json();
  },

  async sendToGrill(staffId: number, customerIdentifier: string, orderType: string, cartItems: CartItem[], subtotal: number, tax: number, total: number): Promise<void> {
    const formattedCart = cartItems.map(i => ({
      prep_item_id: i.prep_item_id,
      qty: i.qty,
      unit_price: i.unit_price,
      pos_display_name: i.pos_display_name,
    }));

    const res = await fetch(`${API_BASE}/pos/send-to-grill`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        staff_id: staffId,
        customer_identifier: customerIdentifier,
        order_type: orderType,
        cart_items: formattedCart,
        subtotal,
        tax,
        total
      })
    });
    if (!res.ok) throw new Error('Failed to send to grill');
  },

  async editActiveOrder(orderId: number, staffId: number, cartItems: CartItem[], subtotal: number, tax: number, total: number): Promise<void> {
    const formattedCart = cartItems.map(i => ({
      prep_item_id: i.prep_item_id,
      qty: i.qty,
      unit_price: i.unit_price,
      pos_display_name: i.pos_display_name,
    }));

    const res = await fetch(`${API_BASE}/pos/edit-order`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        order_id: orderId,
        staff_id: staffId,
        cart_items: formattedCart,
        subtotal,
        tax,
        total
      })
    });
    if (!res.ok) throw new Error('Failed to edit active order');
  },

  async settlePayment(orderId: number, staffId: number): Promise<void> {
    const res = await fetch(`${API_BASE}/pos/settle`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ order_id: orderId, staff_id: staffId })
    });
    if (!res.ok) throw new Error('Failed to settle payment');
  },

  async updateOrderStatus(orderId: number, status: string, staffId: number): Promise<void> {
    const res = await fetch(`${API_BASE}/pos/update-status`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ order_id: orderId, status, staff_id: staffId })
    });
    if (!res.ok) throw new Error('Failed to update status');
  },

  // Add this inside posService in src/services/posService.ts
  async getNextTableNumber(): Promise<number> {
    const res = await fetch(`${API_BASE}/pos/next-table`);
    if (!res.ok) throw new Error('Failed to get next table number');
    return await res.json();
  },

  async reprintReceipt(orderId: number): Promise<void> {
    const res = await fetch(`${API_BASE}/pos/reprint`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ order_id: orderId })
    });
    if (!res.ok) throw new Error('Failed to send reprint command');
  },
};