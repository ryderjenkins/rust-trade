DROP TABLE IF EXISTS tick_data;
CREATE TABLE tick_data (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    volume DOUBLE PRECISION NOT NULL,
    side CHAR(4) NOT NULL CHECK (side IN ('BUY', 'SELL')),
    trade_id VARCHAR(50) NOT NULL,
    is_maker BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tick_data_symbol ON tick_data(symbol);
CREATE INDEX idx_tick_data_timestamp ON tick_data(timestamp);
CREATE INDEX idx_tick_data_symbol_timestamp ON tick_data(symbol, timestamp);
CREATE INDEX idx_tick_data_trade_id ON tick_data(trade_id);