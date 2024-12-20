// src/components/TickerCard.jsx
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

function TickerCard({ symbol }) {
  const [ticker, setTicker] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchTicker = async () => {
      try {
        const response = await invoke('fetch_ticker', { symbol });
        if (response.success) {
          setTicker(response.data);
        }
      } catch (error) {
        console.error('Error fetching ticker:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchTicker();
    const interval = setInterval(fetchTicker, 5000);
    return () => clearInterval(interval);
  }, [symbol]);

  if (loading) return <div>加载中...</div>;

  return ticker && (
    <div className="p-4 border rounded-lg bg-white shadow">
      <h2 className="text-xl font-bold mb-4">{symbol} 行情</h2>
      <div className="grid grid-cols-2 gap-4">
        <div>
          <p className="text-gray-600">最新价格</p>
          <p className="text-2xl font-bold">${ticker.price}</p>
        </div>
        <div>
          <p className="text-gray-600">24h成交量</p>
          <p className="text-lg">{ticker.volume24h}</p>
        </div>
        <div>
          <p className="text-gray-600">24h最高</p>
          <p className="text-green-600">${ticker.high24h}</p>
        </div>
        <div>
          <p className="text-gray-600">24h最低</p>
          <p className="text-red-600">${ticker.low24h}</p>
        </div>
      </div>
    </div>
  );
}
export default TickerCard;