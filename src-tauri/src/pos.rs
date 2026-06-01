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

#[derive(Serialize, sqlx::FromRow)]
pub struct ActiveOrderItemResponse {
    pub prep_item_id: i32,
    pub pos_display_name: String,
    pub qty: i32,
    pub unit_price: f64,
}

#[derive(Deserialize)]
pub struct ReprintReq {
    pub order_id: i32,
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

    // Fetch the detailed items for the receipt
    let items = sqlx::query_as::<_, OrderReceiptItem>(
        "SELECT pi.pos_display_name, oi.quantity, oi.price_at_time_of_sale::float8 FROM order_item oi JOIN prepared_inventory pi ON oi.prep_item_id = pi.prep_item_id WHERE oi.order_id = $1"
    ).bind(payload.order_id).fetch_all(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Fetch order totals and customer identifier
    let order_info: (f64, String) = sqlx::query_as("SELECT total_amount::float8, customer_identifier FROM orders WHERE order_id = $1")
        .bind(payload.order_id).fetch_one(&mut *tx).await.unwrap_or((0.0, "Unknown".to_string()));

    let total = order_info.0;
    let customer = order_info.1;

    let receipt = items.iter().map(|i| format!("- {}x {} (PHP {:.2})", i.quantity, i.pos_display_name, i.price_at_time_of_sale * i.quantity as f64)).collect::<Vec<_>>().join("\n");

    sqlx::query("UPDATE orders SET status = 'Completed' WHERE order_id = $1").bind(payload.order_id).execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    sqlx::query("INSERT INTO system_log (log_category, staff_id, description, details) VALUES ('POS', $1, 'Payment Settled', $2)")
        .bind(payload.staff_id).bind(format!("Order #{} has been paid in full.\n\nOrder Contents:\n{}", payload.order_id, receipt))
        .execute(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // --- NEW: Physical Receipt Printing via ESC/POS Commands ---
    // Runs in a background thread so the UI does not freeze while printing
    tokio::spawn(async move {
        let mut printer_data: Vec<u8> = Vec::new();
        
        // ESC @ (Initialize printer)
        printer_data.extend_from_slice(&[0x1B, 0x40]);
        
        // ESC a 1 (Center align)
        printer_data.extend_from_slice(&[0x1B, 0x61, 0x01]);
        
        printer_data.extend_from_slice(b"BBQ NA MURAG LAMI\n");
        printer_data.extend_from_slice(b"Cagayan De Oro City\n");
        printer_data.extend_from_slice(b"--------------------------------\n");
        
        // ESC a 0 (Left align)
        printer_data.extend_from_slice(&[0x1B, 0x61, 0x00]);
        
        let order_hdr = format!("Identifier: {}\n\n", customer);
        printer_data.extend_from_slice(order_hdr.as_bytes());

        for item in &items {
            let line_total = item.price_at_time_of_sale * item.quantity as f64;
            let name_qty = format!("{}x {}", item.quantity, item.pos_display_name);
            let price_str = format!("{:.2}", line_total);
            
            // Xprinter 58mm fits 32 characters max per line. 
            // We truncate long item names and add spacing to push the price to the right edge.
            let mut safe_name = name_qty.clone();
            if safe_name.len() > 22 {
                safe_name.truncate(22);
            }
            
            let padding = 32_usize.saturating_sub(safe_name.len() + price_str.len());
            let spaces = " ".repeat(padding);
            
            let print_line = format!("{}{}{}\n", safe_name, spaces, price_str);
            printer_data.extend_from_slice(print_line.as_bytes());
        }

        printer_data.extend_from_slice(b"--------------------------------\n");
        
        // ESC a 2 (Right align)
        printer_data.extend_from_slice(&[0x1B, 0x61, 0x02]);
        let total_line = format!("TOTAL: PHP {:.2}\n", total);
        
        // ESC ! 0x11 (Double height & width text for the total)
        printer_data.extend_from_slice(&[0x1B, 0x21, 0x11]);
        printer_data.extend_from_slice(total_line.as_bytes());
        // ESC ! 0x00 (Reset to normal text)
        printer_data.extend_from_slice(&[0x1B, 0x21, 0x00]);

        // ESC a 1 (Center align)
        printer_data.extend_from_slice(&[0x1B, 0x61, 0x01]);
        printer_data.extend_from_slice(b"\nThank you for dining with us!\n\n\n\n\n");
        
        // GS V 0 (Cut paper command)
        printer_data.extend_from_slice(&[0x1D, 0x56, 0x00]);
        
        // ESC p 0 25 250 (Kick open the cash drawer)
        printer_data.extend_from_slice(&[0x1B, 0x70, 0x00, 0x19, 0xFA]);

        // Connect to the Windows Printer Share and send the raw byte array
        let _ = std::fs::write(r"\\localhost\Xprinter", printer_data);
    });

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

pub async fn reprint_receipt(State(pool): State<PgPool>, Json(payload): Json<ReprintReq>) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 1. Fetch items
    let items = sqlx::query_as::<_, OrderReceiptItem>(
        "SELECT pi.pos_display_name, oi.quantity, oi.price_at_time_of_sale::float8 
         FROM order_item oi 
         JOIN prepared_inventory pi ON oi.prep_item_id = pi.prep_item_id 
         WHERE oi.order_id = $1"
    ).bind(payload.order_id).fetch_all(&mut *tx).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 2. Fetch totals
    let order_info: (f64, String) = sqlx::query_as(
        "SELECT total_amount::float8, customer_identifier 
         FROM orders WHERE order_id = $1"
    )
    .bind(payload.order_id).fetch_one(&mut *tx).await.unwrap_or((0.0, "Unknown".to_string()));

    let total = order_info.0;
    let customer = order_info.1;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 3. Build Printer Data (Notice: No more tokio::spawn)
    let mut printer_data: Vec<u8> = Vec::new();
        
    printer_data.extend_from_slice(&[0x1B, 0x40]); // Init
    printer_data.extend_from_slice(&[0x1B, 0x61, 0x01]); // Center
    
    printer_data.extend_from_slice(b"BBQ NA MURAG LAMI\n");
    printer_data.extend_from_slice(b"Cagayan De Oro City\n");
    printer_data.extend_from_slice(b"*** REPRINT ***\n"); 
    printer_data.extend_from_slice(b"--------------------------------\n");
    
    printer_data.extend_from_slice(&[0x1B, 0x61, 0x00]); // Left
    
    let order_hdr = format!("Identifier: {}\n\n", customer);
    printer_data.extend_from_slice(order_hdr.as_bytes());

    for item in &items {
        let line_total = item.price_at_time_of_sale * item.quantity as f64;
        let name_qty = format!("{}x {}", item.quantity, item.pos_display_name);
        let price_str = format!("{:.2}", line_total);
        
        let mut safe_name = name_qty.clone();
        if safe_name.len() > 22 { safe_name.truncate(22); }
        
        let padding = 32_usize.saturating_sub(safe_name.len() + price_str.len());
        let spaces = " ".repeat(padding);
        
        let print_line = format!("{}{}{}\n", safe_name, spaces, price_str);
        printer_data.extend_from_slice(print_line.as_bytes());
    }

    printer_data.extend_from_slice(b"--------------------------------\n");
    
    printer_data.extend_from_slice(&[0x1B, 0x61, 0x02]); // Right
    let total_line = format!("TOTAL: PHP {:.2}\n", total);
    
    printer_data.extend_from_slice(&[0x1B, 0x21, 0x11]); // Double height/width
    printer_data.extend_from_slice(total_line.as_bytes());
    printer_data.extend_from_slice(&[0x1B, 0x21, 0x00]); // Reset

    printer_data.extend_from_slice(&[0x1B, 0x61, 0x01]); // Center
    printer_data.extend_from_slice(b"\nThank you for dining with us!\n\n\n\n\n");
    
    printer_data.extend_from_slice(&[0x1D, 0x56, 0x00]); // Cut
    printer_data.extend_from_slice(&[0x1B, 0x70, 0x00, 0x19, 0xFA]); // Drawer

    // 4. Send to printer using async I/O and catch the error
    if let Err(e) = tokio::fs::write(r"\\localhost\Xprinter", printer_data).await {
        eprintln!("Printer hardware error: {}", e);
        // Return a 500 error so the frontend knows the printer failed
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Printer is offline or disconnected".to_string()));
    }

    Ok(Json(()))
}