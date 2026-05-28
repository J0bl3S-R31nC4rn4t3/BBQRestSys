pub mod models;
pub mod dashboard;
pub mod inventory;
pub mod pos;
pub mod logs;
pub mod staff;
pub mod schedule;
pub mod auth; 
pub mod queue; // NEW: Added queue module

use axum::{routing::{get, post}, Router};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            tauri::async_runtime::spawn(async {
                
                let pool = PgPool::connect("postgres://postgres:nigmagalaxy@localhost/bbq_system")
                    .await.expect("Failed to connect to PostgreSQL");

                sqlx::migrate!("./migrations").run(&pool).await.expect("Failed to run migrations");
                println!("Database connected and migrated.");

                let cors = CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any);

                let dashboard_routes = Router::new()
                    .route("/sales", get(dashboard::get_today_sales))
                    .route("/staff-count", get(dashboard::get_active_staff_count))
                    .route("/low-stock", get(dashboard::get_low_stock_alerts))
                    .route("/top-items", get(dashboard::get_top_selling_items))
                    .route("/metrics", get(dashboard::get_period_metrics));

                let inventory_routes = Router::new()
                    .route("/raw", get(inventory::get_raw_inventory))
                    .route("/prepared", get(inventory::get_prepared_inventory))
                    .route("/edit-stock", post(inventory::edit_stock))
                    .route("/add-raw", post(inventory::add_new_raw_item))
                    .route("/add-prepared", post(inventory::add_prepared_item))
                    .route("/update-pricing", post(inventory::update_prepared_item_pricing))
                    .route("/delete-prepared", post(inventory::delete_prepared_item))
                    .route("/raw-categories", get(inventory::get_available_categories))
                    .route("/parts", get(inventory::get_available_parts))
                    .route("/log-prep", post(inventory::log_prep_transaction))
                    .route("/recent-prep", get(inventory::get_recent_prep_logs))
                    .route("/pos-categories", get(inventory::get_pos_categories))
                    .route("/pos-categories/add", post(inventory::add_pos_category))
                    .route("/pos-categories/remove", post(inventory::remove_pos_category))
                    .route("/upload-photo", post(inventory::upload_photo));

                let pos_routes = Router::new()
                    .route("/active-orders", get(pos::get_active_orders))
                    .route("/send-to-grill", post(pos::send_to_grill))
                    .route("/settle", post(pos::settle_payment))
                    .route("/update-status", post(pos::update_order_status_with_log))
                    .route("/edit-order", post(pos::edit_active_order))
                    .route("/next-table", axum::routing::get(pos::get_next_table_number))
                    .route("/reprint", post(pos::reprint_receipt));

                let schedule_routes = Router::new()
                    .route("/today", get(schedule::get_today_shifts))
                    .route("/active-shift", get(schedule::get_active_shift_for_staff))
                    .route("/clock-in", post(schedule::clock_in))
                    .route("/clock-out", post(schedule::clock_out))
                    .route("/staff/:staff_id", get(schedule::get_staff_shifts)); // FIX: Added the missing route

                let staff_routes = Router::new()
                    .route("/all", get(staff::get_all_staff_full))
                    .route("/create", post(staff::create_staff))
                    .route("/update", post(staff::update_staff))
                    .route("/delete", post(staff::delete_staff))
                    .route("/search", get(staff::search_staff));

                let log_routes = Router::new()
                    .route("/recent", get(logs::get_recent_logs));

                let auth_routes = Router::new()
                    .route("/login", post(auth::verify_login));

                // NEW: Queue Routes for the JSON file
                let queue_routes = Router::new()
                    .route("/", get(queue::get_queue))
                    .route("/next-number", get(queue::get_next_number))
                    .route("/add", post(queue::add_to_queue))
                    .route("/remove/:queue_number", post(queue::remove_from_queue));

                let api_routes = Router::new()
                    .nest("/dashboard", dashboard_routes)
                    .nest("/inventory", inventory_routes)
                    .nest("/pos", pos_routes)
                    .nest("/schedule", schedule_routes)
                    .nest("/staff", staff_routes)
                    .nest("/logs", log_routes)
                    .nest("/auth", auth_routes)
                    .nest("/queue", queue_routes); // NEW: Nested Queue

                let app_router = Router::new()
                    .nest("/api", api_routes)
                    .route("/uploads/:file_name", get(inventory::serve_upload))
                    .with_state(pool)
                    .layer(cors);

                let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
                println!("Axum API Server running on http://0.0.0.0:3000");
                axum::serve(listener, app_router).await.unwrap();
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}