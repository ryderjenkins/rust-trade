import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

function TradesTable({ symbol }) {
  const [trades, setTrades] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchTrades = async () => {
      try {
        const response = await invoke('fetch_trades', { 
          symbol,
          limit: 10
        });
        if (response.success) {
          setTrades(response.data);
        }
      } catch (error) {
        console.error('Error fetching trades:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchTrades();
    const interval = setInterval(fetchTrades, 2000);
    return () => clearInterval(interval);
  }, [symbol]);

  if (loading) return <div>加载中...</div>;

  return (
    <div className="p-4 border rounded-lg bg-white shadow">
      <h2 className="text-xl font-bold mb-4">最近成交</h2>
      <div className="overflow-x-auto">
        <table className="min-w-full">
          <thead>
            <tr className="text-left text-sm">
              <th className="pb-2">价格</th>
              <th className="pb-2">数量</th>
              <th className="pb-2">时间</th>
            </tr>
          </thead>
          <tbody>
            {trades.map((trade, i) => (
              <tr key={i} className="text-sm">
                <td className={trade.is_buyer_maker ? 'text-green-600' : 'text-red-600'}>
                  {trade.price}
                </td>
                <td>{trade.quantity}</td>
                <td>{new Date(trade.timestamp).toLocaleTimeString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
export default TradesTable;