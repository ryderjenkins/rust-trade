# Rust Trading System (rust-trade)
A quantitative trading system written in Rust

[![My Skills](https://skillicons.dev/icons?i=rust,tauri,ts,react,postgresql)](https://skillicons.dev)

[Maybe you don't know much about Tauri.](https://v2.tauri.app/)

Tauri 2.0 is a framework for building lightweight, secure desktop applications using web technologies and Rust. It provides a minimal footprint by leveraging the OS's webview instead of bundling a heavy runtime, offering better performance, security, and native API integration.

## Overview
rust-trade is a quantitative trading system that combines modern trading strategies with artificial intelligence. This software is released under the GNU General Public License v3. In this basic version, I have defined various interfaces. I will expand the functions I hope to achieve in the next step (allowing customers to run their own strategies in this system and see the results of the strategies. At the same time, mint high-quality strategies into NFTs and put them in the blockchain world)

Copyright (C) 2024 Harrison

## How to run

Run in the root directory:

```bash
# Start the development server
cargo tauri dev
```

```bash
# Build the production version
cargo tauri build
```

## Example

![result](assets/version1.png)

## Required Environment Variables
```bash
DATABASE_URL=postgresql://user:password@localhost/dbname
OPENAI_KEY=your-openai-api-key
```

## Development Roadmap

1. **AI Integration Improvements**
   - Implement response caching
   - Add retry mechanism
   - Improve async handling
   - Enhance risk management

2. **Strategy Enhancements**
   - Add more technical indicators
   - Implement hybrid strategies
   - Improve position sizing
   - Add performance metrics

3. **System Optimization**
   - Optimize database queries
   - Improve error handling
   - Add monitoring system
   - Implement data validation

## License
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.