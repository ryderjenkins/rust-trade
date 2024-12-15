# Trading System API Documentation

## Overview
This document describes the REST API endpoints for the trading system. The API provides access to market data including real-time tickers, orderbook information, and historical kline data.

## Base URL
```
http://[host]:[port]/api/v1
```

## Common Response Format
All API endpoints return data in the following JSON format:

```json
{
    "success": boolean,
    "data": object | null,
    "error": string | null
}
```

## Endpoints

### Get Market Ticker
Retrieves the current market ticker information for a specific symbol.

**Endpoint:** `GET /market/ticker/{symbol}`

**Parameters:**
- `symbol` (path parameter, required): Trading pair symbol (e.g., "BTCUSDT")

**Response:**
```json
{
    "success": true,
    "data": {
        "symbol": "string",
        "price": "decimal",
        "timestamp": "ISO8601 timestamp",
        "volume24h": "decimal",
        "high24h": "decimal",
        "low24h": "decimal"
    },
    "error": null
}
```

### Get Orderbook
Retrieves the current orderbook for a specific symbol.

**Endpoint:** `GET /market/orderbook/{symbol}`

**Parameters:**
- `symbol` (path parameter, required): Trading pair symbol (e.g., "BTCUSDT")
- `limit` (query parameter, optional): Number of price levels to return. Default: 20

**Response:**
```json
{
    "success": true,
    "data": {
        "symbol": "string",
        "timestamp": "ISO8601 timestamp",
        "bids": [
            ["price", "quantity"],
            ...
        ],
        "asks": [
            ["price", "quantity"],
            ...
        ]
    },
    "error": null
}
```

### Get Klines (Candlestick Data)
Retrieves historical candlestick data for a specific symbol and time interval.

**Endpoint:** `GET /market/klines`

**Parameters:**
- `symbol` (query parameter, required): Trading pair symbol (e.g., "BTCUSDT")
- `interval` (query parameter, required): Kline interval (e.g., "1m", "5m", "1h", "1d")
- `startTime` (query parameter, optional): Start time in ISO8601 format
- `endTime` (query parameter, optional): End time in ISO8601 format
- `limit` (query parameter, optional): Number of klines to return

**Response:**
```json
{
    "success": true,
    "data": [
        {
            "timestamp": "ISO8601 timestamp",
            "symbol": "string",
            "price": "decimal",
            "volume": "decimal",
            "high": "decimal",
            "low": "decimal",
            "open": "decimal",
            "close": "decimal"
        },
        ...
    ],
    "error": null
}
```

## Error Handling

The API uses standard HTTP status codes and includes error details in the response body when applicable.

**Example Error Response:**
```json
{
    "success": false,
    "data": null,
    "error": "Invalid symbol provided"
}
```

## Rate Limiting
- Implement appropriate rate limiting based on your system's capacity
- Consider adding rate limit headers in responses
- Document specific rate limits when implemented

## Data Types
- All timestamps are in ISO8601 format (e.g., "2024-12-15T10:00:00Z")
- All decimal numbers are strings to preserve precision
- Quantities and prices use decimal format with up to 8 decimal places

## Best Practices
1. Always specify the symbol in uppercase
2. Use appropriate intervals for kline data
3. Include reasonable limit parameters to avoid overloading
4. Handle rate limits appropriately
5. Implement proper error handling for failed requests

## Future Improvements
1. Authentication system for private endpoints
2. WebSocket support for real-time data
3. Additional market data endpoints
4. Enhanced error reporting
5. Pagination for large datasets