# Trading System API Usage Guidelines

Assuming your service is running on: `http://localhost:8080`

## 1. Get market ticker data

Get BTC/USDT market information:

```bash
curl "http://localhost:8080/api/v1/market/ticker/BTCUSDT"
```

Expected response:
```json
{
    "success": true,
    "data": {
        "symbol": "BTCUSDT",
        "price": "42156.85",
        "timestamp": "2024-12-15T10:30:00Z",
        "volume24h": "1234.56",
        "high24h": "42500.00",
        "low24h": "41800.00"
    },
    "error": null
}
```

## 2. Get order book data

### Default depth (20 levels)
```bash
curl "http://localhost:8080/api/v1/market/orderbook/BTCUSDT"
```

### Custom depth (eg: 10 levels)
```bash
curl "http://localhost:8080/api/v1/market/orderbook/BTCUSDT?limit=10"
```

Expected response:
```json
{
    "success": true,
    "data": {
        "symbol": "BTCUSDT",
        "timestamp": "2024-12-15T10:30:05Z",
        "bids": [
            ["42156.85", "1.2345"],
            ["42156.80", "0.5678"]
        ],
        "asks": [
            ["42156.90", "0.8765"],
            ["42156.95", "1.3456"]
        ]
    },
    "error": null
}
```

## 3. Get K-line data

### Get the latest K-line data
```bash
curl "http://localhost:8080/api/v1/market/klines?symbol=BTCUSDT&interval=1h&limit=10"
```

### Get the K-line data of the specified time range
```bash
curl "http://localhost:8080/api/v1/market/klines?symbol=BTCUSDT&interval=1h\
&startTime=2024-12-15T00:00:00Z\
&endTime=2024-12-15T10:00:00Z"
```

Expected response:
```json
{
    "success": true,
    "data": [
        {
            "timestamp": "2024-12-15T09:00:00Z",
            "symbol": "BTCUSDT",
            "price": "42156.85",
            "volume": "123.45",
            "high": "42200.00",
            "low": "42100.00",
            "open": "42150.00",
            "close": "42156.85"
        }
    ],
    "error": null
}
```

## Error response example

When requesting an invalid trading pair:
```bash
curl "http://localhost:8080/api/v1/market/ticker/INVALID"
```

Expected response:
```json
{
    "success": false,
    "data": null,
    "error": "Invalid symbol provided"
}
```

## Usage tips

1. **Check connectivity**
```bash
# Use the -v parameter to view detailed request/response information
curl -v "http://localhost:8080/api/v1/market/ticker/BTCUSDT"
```

2. **Format JSON output**
```bash
# Use jq tool to beautify the output (need to install jq first)
curl "http://localhost:8080/api/v1/market/ticker/BTCUSDT" | jq '.'
```

3. **Test K-line data at different time intervals**
```bash
# 1 minute K-line
curl "http://localhost:8080/api/v1/market/klines?symbol=BTCUSDT&interval=1m&limit=5"

# 1 hour K-line
curl "http://localhost:8080/api/v1/market/klines?symbol=BTCUSDT&interval=1h&limit=5"

# 1 day K-line
curl "http://localhost:8080/api/v1/market/klines?symbol=BTCUSDT&interval=1d&limit=5"
```

## Common Problems Troubleshooting

1. If you encounter a connection error, please check:
- Is the service running?
- Is the port number correct?
- Firewall settings

2. If you receive a 404 error, please check:
- Is the URL path correct?
- Is the transaction pair name correct (case sensitive)

3. If the data is abnormal, please check:
- Is the time parameter format correct
- Is the limit parameter within a reasonable range