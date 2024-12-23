// src/app/page.tsx
'use client';

import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { format } from 'date-fns';
import { Loader2 } from 'lucide-react';

interface MarketDataPoint {
  timestamp: string;
  symbol: string;
  price: number;
  volume: number;
  high: number;
  low: number;
  open: number;
  close: number;
}

interface MarketOverview {
  price: number;
  price_change_24h: number;
  volume_24h: number;
}

const intervals = [
  { label: '1m', value: '1m' },
  { label: '5m', value: '5m' },
  { label: '15m', value: '15m' },
  { label: '1h', value: '1h' },
  { label: '4h', value: '4h' },
  { label: '1d', value: '1d' },
];

export default function MarketDashboard() {
  const [symbol] = useState('BTCUSDT');
  const [candlesticks, setCandlesticks] = useState<MarketDataPoint[]>([]);
  const [marketOverview, setMarketOverview] = useState<MarketOverview | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeInterval, setActiveInterval] = useState('1d');

  const fetchMarketOverview = useCallback(async () => {
    try {
      const overview = await invoke<MarketOverview>('get_market_overview', { symbol });
      setMarketOverview(overview);
    } catch (err) {
      console.error('Error fetching market overview:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch market overview');
    }
  }, [symbol]);

  const fetchCandlesticks = useCallback(
    async (startTime?: string, endTime?: string) => {
      try {
        const data = await invoke<MarketDataPoint[]>('get_candlestick_data', {
          symbol,
          interval: activeInterval,
          start_time: startTime,
          end_time: endTime,
          limit: 100,
        });

        if (data.length === 0) {
          console.log('No data found for the selected time range, trying previous range...');
          // 尝试一个稍早的时间区间
          const { startTime: fallbackStartTime, endTime: fallbackEndTime } = getStartEndTimeForInterval(getPreviousInterval(activeInterval));
          return fetchCandlesticks(fallbackStartTime, fallbackEndTime);
        }

        const sortedData = data.sort(
          (a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
        );
        setCandlesticks(sortedData);
        setError(null);
      } catch (err) {
        console.error('Error fetching candlesticks:', err);
        setError(err instanceof Error ? err.message : 'Failed to fetch candlestick data');
      }
    },
    [symbol, activeInterval]
  );
  
  useEffect(() => {
    const fetchData = async () => {
      setIsLoading(true);
      setError(null);
      try {
        await Promise.all([fetchMarketOverview(), fetchCandlesticks()]); 
      } catch (err) {
        console.error('Error fetching data:', err);
        setError(err instanceof Error ? err.message : 'Failed to fetch data');
      } finally {
        setIsLoading(false);
      }
    };
  
    fetchData();
    const intervalId = window.setInterval(fetchData, 60000); 
    return () => {
      window.clearInterval(intervalId); 
    };
  }, [fetchMarketOverview, fetchCandlesticks]);
  

  const handleIntervalChange = async (newInterval: string) => {
    setActiveInterval(newInterval);
    setIsLoading(true);
    try {
      const { startTime, endTime } = getStartEndTimeForInterval(newInterval);
      await fetchCandlesticks(startTime, endTime);
    } catch (err) {
      console.error('Error fetching candlesticks:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch candlestick data');
    } finally {
      setIsLoading(false);
    }
  };

  // 获取上一个时间区间，例如，如果当前选择了1m，则回退到5m
  const getPreviousInterval = (interval: string) => {
    const intervalOrder = ['1m', '5m', '15m', '1h', '4h', '1d'];
    const currentIndex = intervalOrder.indexOf(interval);
    return currentIndex > 0 ? intervalOrder[currentIndex - 1] : interval;
  };
  
  const getStartEndTimeForInterval = (interval: string) => {
    const now = new Date();
    let startTime = now;
    switch (interval) {
      case '1m':
        startTime = new Date(now.getTime() - 1 * 60000); 
        break;
      case '5m':
        startTime = new Date(now.getTime() - 5 * 60000);
        break;
      case '15m':
        startTime = new Date(now.getTime() - 15 * 60000);
        break;
      case '1h':
        startTime = new Date(now.getTime() - 60 * 60000);
        break;
      case '4h':
        startTime = new Date(now.getTime() - 4 * 60 * 60000);
        break;
      case '1d':
        startTime = new Date(now.getTime() - 24 * 60 * 60000);
        break;
      default:
        startTime = now;
        break;
    }
    return { startTime: startTime.toISOString(), endTime: now.toISOString() };
  };  

  if (error) {
    return (
      <div className="p-4 text-red-500 bg-red-100 rounded">
        Error: {error}
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  return (
    <div className="p-6">
      {/* Market Overview */}
      <div className="mb-8 bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
        <h2 className="text-xl font-bold mb-4">Market Overview</h2>
        {marketOverview && (
          <div className="grid grid-cols-3 gap-4">
            <div>
              <p className="text-sm text-gray-500">Price</p>
              <p className="text-2xl font-bold">
                ${marketOverview.price.toLocaleString(undefined, { minimumFractionDigits: 2 })}
              </p>
            </div>
            <div>
              <p className="text-sm text-gray-500">24h Change</p>
              <p className={`text-lg font-semibold ${
                marketOverview.price_change_24h >= 0 ? 'text-green-500' : 'text-red-500'
              }`}>
                {marketOverview.price_change_24h.toFixed(2)}%
              </p>
            </div>
            <div>
              <p className="text-sm text-gray-500">24h Volume</p>
              <p className="text-lg font-semibold">
                ${marketOverview.volume_24h.toLocaleString(undefined, { maximumFractionDigits: 0 })}
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Interval Selector */}
      <div className="mb-4 flex space-x-2">
        {intervals.map((int) => (
          <button
            key={int.value}
            onClick={() => handleIntervalChange(int.value)}
            className={`px-4 py-2 rounded ${
              activeInterval === int.value
                ? 'bg-blue-500 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            }`}
          >
            {int.label}
          </button>
        ))}
      </div>

      {/* 左侧导航按钮处理 */}
      <div className="fixed left-0 top-0 h-full w-64 bg-gray-900 p-4">
        <nav className="space-y-4">
          <button
            onClick={() => window.location.href = '/dashboard'}
            className="text-white hover:text-blue-400 block w-full text-left px-4 py-2 rounded hover:bg-gray-800"
          >
            Dashboard
          </button>
          <button
            onClick={() => window.location.href = '/trading'}
            className="text-white hover:text-blue-400 block w-full text-left px-4 py-2 rounded hover:bg-gray-800"
          >
            Trading
          </button>
          <button
            onClick={() => window.location.href = '/backtest'}
            className="text-white hover:text-blue-400 block w-full text-left px-4 py-2 rounded hover:bg-gray-800"
          >
            Backtest
          </button>
          <button
            onClick={() => window.location.href = '/settings'}
            className="text-white hover:text-blue-400 block w-full text-left px-4 py-2 rounded hover:bg-gray-800"
          >
            Settings
          </button>
        </nav>
      </div>

      {/* Price Chart */}
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
        <h2 className="text-xl font-bold mb-4">Price Chart</h2>
        <div className="h-96">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart data={candlesticks}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis 
                dataKey="timestamp" 
                tickFormatter={(time) => format(new Date(time), 'HH:mm')}
              />
              <YAxis domain={['auto', 'auto']} />
              <Tooltip
                labelFormatter={(label) => format(new Date(label), 'yyyy-MM-dd HH:mm')}
                formatter={(value: number) => [
                  `$${value.toLocaleString(undefined, { minimumFractionDigits: 2 })}`,
                  'Price'
                ]}
              />
              <Line 
                type="monotone" 
                dataKey="close" 
                stroke="#2563eb" 
                dot={false}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      {/* Market Details */}
      <div className="mt-8 grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
          <h2 className="text-xl font-bold mb-4">Trade History</h2>
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="text-left border-b">
                  <th className="pb-2">Time</th>
                  <th className="pb-2">Price</th>
                  <th className="pb-2">Volume</th>
                </tr>
              </thead>
              <tbody>
                {candlesticks.slice(0, 10).map((candle) => (
                  <tr key={candle.timestamp} className="border-b">
                    <td className="py-2">{format(new Date(candle.timestamp), 'HH:mm:ss')}</td>
                    <td className="py-2">${candle.close.toLocaleString()}</td>
                    <td className="py-2">{candle.volume.toLocaleString()}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-6">
          <h2 className="text-xl font-bold mb-4">Market Statistics</h2>
          {candlesticks.length > 0 && (
            <div className="grid grid-cols-2 gap-4">
              <div>
                <p className="text-sm text-gray-500">High</p>
                <p className="text-lg font-semibold">
                  ${Math.max(...candlesticks.map(c => c.high)).toLocaleString()}
                </p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Low</p>
                <p className="text-lg font-semibold">
                  ${Math.min(...candlesticks.map(c => c.low)).toLocaleString()}
                </p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Open</p>
                <p className="text-lg font-semibold">
                  ${candlesticks[candlesticks.length - 1].open.toLocaleString()}
                </p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Close</p>
                <p className="text-lg font-semibold">
                  ${candlesticks[0].close.toLocaleString()}
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}