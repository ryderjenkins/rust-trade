# Rust Trading System (rust-trade)
A quantitative trading system written in Rust

[![My Skills](https://skillicons.dev/icons?i=rust,tauri,ts,react,postgresql)](https://skillicons.dev)

[Maybe you don't know much about Tauri.](https://v2.tauri.app/)

Tauri 2.0 is a framework for building lightweight, secure desktop applications using web technologies and Rust. It provides a minimal footprint by leveraging the OS's webview instead of bundling a heavy runtime, offering better performance, security, and native API integration.

## Overview
rust-trade is a quantitative trading system that combines modern trading strategies with artificial intelligence. This software is released under the GNU General Public License v3.

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

![result](assets/version2.png)

## Required Environment Variables
```bash
DATABASE_URL=postgresql://user:password@localhost/dbname
```

## Development Roadmap

1. Add more strategy templates
2. Implement strategy scoring system
3. Develop strategy market function
4. Add real trading support
5. Optimize performance indicator calculation
6. Add more data analysis tools

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