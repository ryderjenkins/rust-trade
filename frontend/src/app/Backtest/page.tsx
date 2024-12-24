'use client';

import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

interface BacktestParams {
  symbol: string;
  days: number;
  initialCapital: string;
  commissionRate: string;
  shortPeriod: number;
  longPeriod: number;
}

// 后端返回的交易记录格式
interface TradeResponse {
  timestamp: string;
  side: 'Buy' | 'Sell';
  symbol: string;
  quantity: string;
  price: string;
  commission: string;
}

// 权益曲线数据点格式
interface EquityPoint {
  timestamp: string;
  value: string;
}

// 后端返回的完整回测结果格式
interface BacktestResponse {
  equity_curve: EquityPoint[];
  losing_trades: number;
  max_drawdown: string;
  total_return: string;
  total_trades: number;
  trades: TradeResponse[];
  winning_trades: number;
}

export default function Backtest() {
  const [params, setParams] = useState<BacktestParams>({
    symbol: 'BTCUSDT',
    days: 30,
    initialCapital: '10000',
    commissionRate: '0.001',
    shortPeriod: 5,
    longPeriod: 20,
  });
  const [result, setResult] = useState<BacktestResponse | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 添加格式化函数
  const formatPercentage = (value: string) => {
    const num = parseFloat(value);
    return num.toFixed(2);
  };

  const formatPrice = (value: string) => {
    const num = parseFloat(value);
    return num.toFixed(2);
  };

  const formatQuantity = (value: string) => {
    const num = parseFloat(value);
    return num.toFixed(8); // 保留8位小数，适合加密货币数量
  };

  const formatCommission = (value: string) => {
    const num = parseFloat(value);
    return num.toFixed(4); // 佣金保留4位小数
  };

  const runBacktest = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const response = await invoke<BacktestResponse>('run_backtest', {
        symbol: params.symbol,
        days: params.days,
        initialCapital: params.initialCapital,
        commissionRate: params.commissionRate,
        shortPeriod: params.shortPeriod,
        longPeriod: params.longPeriod,
      });

      console.log('Raw backtest response:', response);
      setResult(response);
    } catch (err) {
      console.error('Backtest error:', err);
      setError(err instanceof Error ? err.message : 'Failed to run backtest');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-6">Backtest Strategy</h1>
      
      {/* Parameters Form */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6 mb-6">
        <h2 className="text-xl font-semibold mb-4">Parameters</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div>
            <label className="block text-sm font-medium mb-1">Symbol</label>
            <input
              type="text"
              value={params.symbol}
              onChange={(e) => setParams({ ...params, symbol: e.target.value })}
              className="w-full p-2 border rounded"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Days</label>
            <input
              type="number"
              value={params.days}
              onChange={(e) => setParams({ ...params, days: parseInt(e.target.value) })}
              className="w-full p-2 border rounded"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Initial Capital</label>
            <input
              type="text"
              value={params.initialCapital}
              onChange={(e) => setParams({ ...params, initialCapital: e.target.value })}
              className="w-full p-2 border rounded"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Commission Rate</label>
            <input
              type="text"
              value={params.commissionRate}
              onChange={(e) => setParams({ ...params, commissionRate: e.target.value })}
              className="w-full p-2 border rounded"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Short Period</label>
            <input
              type="number"
              value={params.shortPeriod}
              onChange={(e) => setParams({ ...params, shortPeriod: parseInt(e.target.value) })}
              className="w-full p-2 border rounded"
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Long Period</label>
            <input
              type="number"
              value={params.longPeriod}
              onChange={(e) => setParams({ ...params, longPeriod: parseInt(e.target.value) })}
              className="w-full p-2 border rounded"
            />
          </div>
        </div>
        <button
          onClick={runBacktest}
          disabled={isLoading}
          className="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-gray-400"
        >
          {isLoading ? 'Running...' : 'Run Backtest'}
        </button>
      </div>

      {error && (
        <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-6">
          {error}
        </div>
      )}

      {/* Results Summary */}
      {result && (
        <div className="space-y-6">
          <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <h2 className="text-xl font-semibold mb-4">Backtest Results</h2>
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
              <div>
                <p className="text-sm text-gray-500">Total Return</p>
                <p className="text-xl font-bold">{formatPercentage(result.total_return)}%</p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Total Trades</p>
                <p className="text-xl font-bold">{result.total_trades}</p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Winning Trades</p>
                <p className="text-xl font-bold">{result.winning_trades}</p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Losing Trades</p>
                <p className="text-xl font-bold">{result.losing_trades}</p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Max Drawdown</p>
                <p className="text-xl font-bold">{formatPercentage(result.max_drawdown)}%</p>
              </div>
            </div>
          </div>

          {/* Equity Curve */}
          {result.equity_curve && result.equity_curve.length > 0 && (
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
              <h2 className="text-xl font-semibold mb-4">Equity Curve</h2>
              <div className="h-96">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart 
                    data={result.equity_curve.map(point => ({
                      timestamp: new Date(point.timestamp).getTime(),
                      value: parseFloat(point.value)
                    }))}
                  >
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis 
                      dataKey="timestamp"
                      type="number"
                      domain={['auto', 'auto']}
                      tickFormatter={(timestamp) => new Date(timestamp).toLocaleDateString()}
                      scale="time"
                    />
                    <YAxis 
                      domain={['auto', 'auto']}
                      tickFormatter={(value) => `$${value.toFixed(2)}`}
                    />
                    <Tooltip 
                      labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()}
                      formatter={(value: number) => [`$${value.toFixed(2)}`, 'Portfolio Value']}
                    />
                    <Line 
                      type="monotone" 
                      dataKey="value" 
                      stroke="#2563eb" 
                      dot={false}
                      isAnimationActive={false}
                    />
                  </LineChart>
                </ResponsiveContainer>
              </div>
            </div>
          )}

          {/* Trade History */}
          {result.trades && result.trades.length > 0 && (
            <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
              <h2 className="text-xl font-semibold mb-4">Trade History</h2>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="text-left border-b">
                      <th className="pb-2">Time</th>
                      <th className="pb-2">Side</th>
                      <th className="pb-2">Symbol</th>
                      <th className="pb-2">Quantity</th>
                      <th className="pb-2">Price</th>
                      <th className="pb-2">Commission</th>
                    </tr>
                  </thead>
                  <tbody>
                    {result.trades.map((trade, index) => (
                      <tr key={index} className="border-b">
                        <td className="py-2">{new Date(trade.timestamp).toLocaleString()}</td>
                        <td className={`py-2 ${trade.side === 'Buy' ? 'text-green-500' : 'text-red-500'}`}>
                          {trade.side}
                        </td>
                        <td className="py-2">{trade.symbol}</td>
                        <td className="py-2">{formatQuantity(trade.quantity)}</td>
                        <td className="py-2">${formatPrice(trade.price)}</td>
                        <td className="py-2">${formatCommission(trade.commission)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}