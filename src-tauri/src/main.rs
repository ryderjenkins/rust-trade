// rust-trade: A quantitative trading system written in Rust
// Copyright (C) 2024 Harrison
//
// This program is part of rust-trade and is released under the
// GNU GPL v3 or later. See the LICENSE file for details.

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod commands;
mod state;

use commands::{get_market_data, get_latest_price, get_candlestick_data,get_market_overview, run_backtest};
use state::AppState;

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
          get_candlestick_data,
          run_backtest
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