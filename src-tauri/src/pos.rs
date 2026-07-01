use axum::{extract::State, Json, http::StatusCode};
use sqlx::PgPool;
use serde::{Deserialize, Serialize}; 
use crate::models::*;

#[derive(Deserialize)]
pub struct SendToGrillReq { 
    pub staff_id: i32, 
    pub customer_identifier: String, 
    pub order_type: String, 
    pub cart_items: Vec<SendToGrillItem>, 
    pub subtotal: f64, 
    pub tax: f64, 
    pub total: f64 
}

#[derive(Deserialize)]
pub struct OrderStaffReq { 
    pub order_id: i32, 
    pub staff_id: i32 
}

#[derive(Deserialize)]
pub struct UpdateStatusReq { 
    pub order_id: i32, 
    pub status: String, 
    pub staff_id: i32 
}

#[derive(Deserialize)]
pub struct EditOrderReq {
    pub order_id: i32,
    pub staff_id: i32,
    pub cart_items: Vec<SendToGrillItem>,
    pub subtotal: f64,
    pub tax: f64,
    pub total: f64,
}

#[derive(Serialize)]
pub struct ActiveOrderResponse {
    pub order_id: i32,
    pub customer_identifier: String,
    pub order_type: String,
    pub total_amount: f64,
    pub status: String,
    pub timestamp: String,
    pub cart_items: Vec<ActiveOrderItemResponse>, 
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct ActiveOrderItemResponse {
    pub prep_item_id: i32,
    pub pos_display_name: String,
    pub qty: i32,
    pub unit_price: f64,
}

#[derive(Deserialize)]
pub struct ReprintReq {
    pub order_id: i32,
    pub customer_identifier: String,
    pub order_type: String,
    pub total_amount: f64,
    pub status: String,
    pub timestamp: String,
    pub cart_items: Vec<ActiveOrderItemResponse>,
}

pub async fn get_active_orders(State(pool): State<PgPool>) -> AppResult<Vec<ActiveOrderResponse>> {
    let orders = sqlx::query_as::<_, ActiveOrder>(
        "SELECT order_id, customer_identifier, order_type, total_amount::float8, status, timestamp
         FROM orders WHERE status != 'Completed' AND status != 'Pending PIN' ORDER BY timestamp ASC"
    ).fetch_all(&pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let mut full_orders = Vec::new();

    for order in orders {
        let items = sqlx::query_as::<_, ActiveOrderItemResponse>(
            "SELECT pi.prep_item_id, pi.pos_display_name, oi.quantity as qty, oi.price_at_time_of_sale::float8 as unit_price 
             FROM order_item oi 
             JOIN prepared_inventory pi ON oi.prep_item_id = pi.prep_item_id 
             WHERE oi.order_id = $1"
        )
        .bind(order.order_id)
        .fetch_all(&pool).await.unwrap_or_default();

        full_orders.push(ActiveOrderResponse {
            order_id: order.order_id,
            customer_identifier: order.customer_identifier,
            order_type: order.order_type,
            total_amount: order.total_amount,
            status: order.status.unwrap_or_default(), 
            timestamp: order.timestamp.to_string(), 
            cart_items: items,
        });
    }
    
    Ok(Json(full_orders))
}

pub async fn send_to_grill(State(pool): State<PgPool>, Json(payload): Json<SendToGrillReq>) -> AppResult<i32> {
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let row: (i32,) = sqlx::query_as(
        "INSERT INTO orders (staff_id, customer_identifier, order_type, subtotal, tax_amount, total_amount, status)
         VALUES ($1, $2, $3, $4::numeric, $5::numeric, $6::numeric, 'Cooking') RETURNING order_id"
    )
    .bind(payload.staff_id).bind(&payload.customer_identifier).bind(&payload.order_type)
    .bind(payload.subtotal).bind(payload.tax).bind(payload.total)
    .fetch_one(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let order_id = row.0;
    let mut itemized = String::new();

    for item in &payload.cart_items {
        sqlx::query("INSERT INTO order_item (order_id, prep_item_id, quantity, price_at_time_of_sale) VALUES ($1, $2, $3, $4::numeric)")
            .bind(order_id).bind(item.prep_item_id).bind(item.qty).bind(item.unit_price)
            .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        sqlx::query("UPDATE prepared_inventory SET current_stock_pieces = current_stock_pieces - $1 WHERE prep_item_id = $2")
            .bind(item.qty).bind(item.prep_item_id)
            .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        itemized.push_str(&format!("- {}x {} (PHP {:.2})\n", item.qty, item.pos_display_name, item.unit_price * item.qty as f64));
    }

    let details = format!("Order Type: {}\nCustomer/Table: {}\n\nItems Ordered:\n{}\nTotal Amount: PHP {:.2}", payload.order_type, payload.customer_identifier, itemized, payload.total);
    sqlx::query("INSERT INTO system_log (log_category, staff_id, description, details) VALUES ('POS', $1, 'Order Sent to Grill', $2)")
        .bind(payload.staff_id).bind(details).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(order_id))
}

pub async fn edit_active_order(State(pool): State<PgPool>, Json(payload): Json<EditOrderReq>) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let old_items: Vec<(i32, i32)> = sqlx::query_as("SELECT prep_item_id, quantity FROM order_item WHERE order_id = $1")
        .bind(payload.order_id).fetch_all(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for (prep_id, qty) in old_items {
        sqlx::query("UPDATE prepared_inventory SET current_stock_pieces = current_stock_pieces + $1 WHERE prep_item_id = $2")
            .bind(qty).bind(prep_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    sqlx::query("DELETE FROM order_item WHERE order_id = $1")
        .bind(payload.order_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("UPDATE orders SET subtotal = $1::numeric, tax_amount = $2::numeric, total_amount = $3::numeric WHERE order_id = $4")
        .bind(payload.subtotal).bind(payload.tax).bind(payload.total).bind(payload.order_id)
        .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut itemized = String::new();
    for item in &payload.cart_items {
        sqlx::query("INSERT INTO order_item (order_id, prep_item_id, quantity, price_at_time_of_sale) VALUES ($1, $2, $3, $4::numeric)")
            .bind(payload.order_id).bind(item.prep_item_id).bind(item.qty).bind(item.unit_price)
            .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        sqlx::query("UPDATE prepared_inventory SET current_stock_pieces = current_stock_pieces - $1 WHERE prep_item_id = $2")
            .bind(item.qty).bind(item.prep_item_id)
            .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        itemized.push_str(&format!("- {}x {} (PHP {:.2})\n", item.qty, item.pos_display_name, item.unit_price * item.qty as f64));
    }

    let details = format!("Order #{} edited.\nNew Total: PHP {:.2}\nNew Items:\n{}", payload.order_id, payload.total, itemized);
    sqlx::query("INSERT INTO system_log (log_category, staff_id, description, details) VALUES ('POS', $1, 'Active Order Edited', $2)")
        .bind(payload.staff_id).bind(details).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(()))
}

pub async fn settle_payment(State(pool): State<PgPool>, Json(payload): Json<OrderStaffReq>) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let items = sqlx::query_as::<_, OrderReceiptItem>(
        "SELECT pi.pos_display_name, oi.quantity, oi.price_at_time_of_sale::float8 FROM order_item oi JOIN prepared_inventory pi ON oi.prep_item_id = pi.prep_item_id WHERE oi.order_id = $1"
    ).bind(payload.order_id).fetch_all(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let receipt = items.iter().map(|i| format!("- {}x {} (PHP {:.2})", i.quantity, i.pos_display_name, i.price_at_time_of_sale * i.quantity as f64)).collect::<Vec<_>>().join("\n");

    sqlx::query("UPDATE orders SET status = 'Completed' WHERE order_id = $1")
        .bind(payload.order_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    sqlx::query("INSERT INTO system_log (log_category, staff_id, description, details) VALUES ('POS', $1, 'Payment Settled', $2)")
        .bind(payload.staff_id).bind(format!("Order #{} has been paid in full.\n\nOrder Contents:\n{}", payload.order_id, receipt))
        .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // The tokio::spawn block for hardware printing has been removed.
    
    Ok(Json(()))
}

pub async fn update_order_status_with_log(State(pool): State<PgPool>, Json(payload): Json<UpdateStatusReq>) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let items = sqlx::query_as::<_, OrderReceiptItem>(
        "SELECT pi.pos_display_name, oi.quantity, oi.price_at_time_of_sale::float8 FROM order_item oi JOIN prepared_inventory pi ON oi.prep_item_id = pi.prep_item_id WHERE oi.order_id = $1"
    ).bind(payload.order_id).fetch_all(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let receipt = items.iter().map(|i| format!("- {}x {} (PHP {:.2})", i.quantity, i.pos_display_name, i.price_at_time_of_sale * i.quantity as f64)).collect::<Vec<_>>().join("\n");

    sqlx::query("UPDATE orders SET status = $1 WHERE order_id = $2").bind(&payload.status).bind(payload.order_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query("INSERT INTO system_log (log_category, staff_id, description, details) VALUES ('POS', $1, 'Order Status Updated', $2)")
        .bind(payload.staff_id).bind(format!("Order #{} marked as {}\n\nOrder Contents:\n{}", payload.order_id, payload.status, receipt))
        .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(()))
}

pub async fn get_next_table_number(State(pool): State<PgPool>) -> Result<Json<i32>, (StatusCode, String)> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT customer_identifier FROM orders 
         WHERE order_type = 'Dine-in' AND customer_identifier LIKE '%Table %' 
         ORDER BY timestamp DESC LIMIT 1"
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut next_table = 1;

    if let Some((identifier,)) = row {
        if let Some(table_str) = identifier.split("Table ").last() {
            if let Ok(current_table) = table_str.trim().parse::<i32>() {
                next_table = if current_table >= 100 { 1 } else { current_table + 1 };
            }
        }
    }

    Ok(Json(next_table))
}

pub async fn reprint_receipt(State(_pool): State<PgPool>, Json(payload): Json<ReprintReq>) -> AppResult<()> {
    
    // We no longer need database queries. We build the printer data directly from the payload.
    let mut printer_data: Vec<u8> = Vec::new();
        
    // Init & Center Align
    printer_data.extend_from_slice(&[0x1B, 0x40]);
    printer_data.extend_from_slice(&[0x1B, 0x61, 0x01]); 
    
    // Header (Matching your Vue layout)
    printer_data.extend_from_slice(b"BBQ NA MURAG LAMI\n");
    printer_data.extend_from_slice(b"Ampayon, Butuan City\n\n"); 
    
    // Left Align for Details
    printer_data.extend_from_slice(&[0x1B, 0x61, 0x00]); 
    
    // Formatted Order Metadata (Strict 32 character math)
    // "Order #:" = 8 chars, leaves 24 chars for the value
    // "Type:" = 5 chars, leaves 27 chars for the value
    // "Date:" = 5 chars, leaves 27 chars for the value
    let order_meta = format!(
        "Order #:{:>24}\nType:{:>27}\nDate:{:>27}\n\nCustomer/Table:\n{}\n",
        payload.order_id, 
        payload.order_type, 
        payload.timestamp, 
        payload.customer_identifier
    );
    printer_data.extend_from_slice(order_meta.as_bytes());

    // Table Headers
    printer_data.extend_from_slice(b"--------------------------------\n");
    printer_data.extend_from_slice(b"QTY ITEM                     AMT\n");
    printer_data.extend_from_slice(b"--------------------------------\n");

    let mut item_count = 0;

    // Line Items (Smart Wrapping Logic)
    for item in &payload.cart_items {
        item_count += item.qty;
        let line_total = item.unit_price * item.qty as f64;
        
        let qty_str = format!("{:<4}", item.qty); // 4 chars reserved for quantity
        let price_str = format!("{:.2}", line_total);
        let name = item.pos_display_name.clone();
        
        // Calculate the maximum characters the name can take to fit on ONE line
        // 32 total chars minus QTY length minus Price length minus 1 guaranteed space
        let max_single_line_name_len = 32_usize.saturating_sub(qty_str.len() + price_str.len() + 1);

        if name.len() <= max_single_line_name_len {
            // The name fits perfectly on a single line
            let padding_length = 32_usize.saturating_sub(qty_str.len() + name.len() + price_str.len());
            let spaces = " ".repeat(padding_length);
            
            let print_line = format!("{}{}{}{}\n", qty_str, name, spaces, price_str);
            printer_data.extend_from_slice(print_line.as_bytes());
        } else {
            // The name is too long! Print the full name on Line 1, and drop the price to Line 2
            let mut safe_name = name;
            
            // 28 is the absolute max name length on line 1 (32 total - 4 for QTY column)
            if safe_name.len() > 28 { safe_name.truncate(28); }
            
            // Line 1: Quantity and Item Name
            let line1 = format!("{}{}\n", qty_str, safe_name);
            
            // Line 2: Empty spaces pushing the price to the far right edge
            let line2_spaces = " ".repeat(32_usize.saturating_sub(price_str.len()));
            let line2 = format!("{}{}\n", line2_spaces, price_str);
            
            printer_data.extend_from_slice(line1.as_bytes());
            printer_data.extend_from_slice(line2.as_bytes());
        }
    }

    printer_data.extend_from_slice(b"--------------------------------\n");
    
    // Right Align for Total
    printer_data.extend_from_slice(&[0x1B, 0x61, 0x02]); 
    
    // Emphasized Total
    let total_line = format!("TOTAL: PHP {:.2}\n", payload.total_amount);
    printer_data.extend_from_slice(&[0x1B, 0x21, 0x11]); // Double height/width
    printer_data.extend_from_slice(total_line.as_bytes());
    printer_data.extend_from_slice(&[0x1B, 0x21, 0x00]); // Reset text size
    
    // Item Count under Total
    let count_line = format!("Item Count: {}\n", item_count);
    printer_data.extend_from_slice(count_line.as_bytes());

    // Center Align for Footer
    printer_data.extend_from_slice(&[0x1B, 0x61, 0x01]); 
    printer_data.extend_from_slice(b"\nThank you for dining with us!\nPlease come again.\n\n\n\n\n");
    
    // Cut Paper & Kick Drawer
    printer_data.extend_from_slice(&[0x1D, 0x56, 0x00]); 
    printer_data.extend_from_slice(&[0x1B, 0x70, 0x00, 0x19, 0xFA]); 

    // Send to Windows Spooler
    if let Err(e) = tokio::fs::write(r"\\localhost\POS-58 11.3.0.1", printer_data).await {
        eprintln!("Printer hardware error: {}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Printer is offline or disconnected".to_string()));
    }

    Ok(Json(()))
}