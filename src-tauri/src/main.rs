#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod commands;
mod state;

use commands::market::{get_market_data, get_latest_price, get_candlestick_data,get_market_overview};
use state::AppState;
use tauri::Manager;

fn main() {
  // 初始化日志
  tracing_subscriber::fmt::init();
  
  // 创建运行时
  let runtime = tokio::runtime::Runtime::new()
      .expect("Failed to create Tokio runtime");
  
  // 在运行时中初始化状态
  let app_state = runtime.block_on(async {
      AppState::new()
          .await
          .expect("Failed to initialize app state")
  });

  tauri::Builder::default()
      .manage(app_state)
      .invoke_handler(tauri::generate_handler![
          get_market_data,
          get_latest_price,
          get_market_overview,
          get_candlestick_data
      ])
      .setup(|app| {
          #[cfg(debug_assertions)]
          {
              let app_handle = app.handle();
              app_handle.plugin(
                  tauri_plugin_shell::init()
              )?;
          }
          Ok(())
      })
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}