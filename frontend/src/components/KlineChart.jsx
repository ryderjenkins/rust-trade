import { useState, useEffect } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from "recharts";

function KlineChart({ symbol }) {
  const [klines, setKlines] = useState([]);
  const [loading, setLoading] = useState(true);
  const [interval, setInterval] = useState('1h');
  const [timeRange, setTimeRange] = useState('1d');

  useEffect(() => {
    const fetchKlines = async () => {
      try {
        const endTime = new Date();
        const startTime = new Date();
        switch (timeRange) {
          case '1d':
            startTime.setDate(startTime.getDate() - 1);
            break;
          case '1w':
            startTime.setDate(startTime.getDate() - 7);
            break;
          case '1m':
            startTime.setMonth(startTime.getMonth() - 1);
            break;
          default:
            startTime.setDate(startTime.getDate() - 1);
        }

        const response = await fetch(
          `http://localhost:8080/api/v1/market/klines?symbol=${symbol}&interval=${interval}&startTime=${startTime.toISOString()}&endTime=${endTime.toISOString()}`
        );
        const data = await response.json();
        if (data.success) {
          setKlines(data.data);
        }
      } catch (error) {
        console.error('Error fetching klines:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchKlines();
    const timer = setInterval(fetchKlines, 60000);
    return () => clearInterval(timer);
  }, [symbol, interval, timeRange]);

  if (loading) return <div>加载中...</div>;

  return (
    <div className="p-4 border rounded-lg bg-white shadow">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-xl font-bold">K线图表</h2>
        <div className="flex gap-4">
          <select
            value={interval}
            onChange={(e) => setInterval(e.target.value)}
            className="border rounded px-2 py-1"
          >
            <option value="1m">1分钟</option>
            <option value="5m">5分钟</option>
            <option value="15m">15分钟</option>
            <option value="1h">1小时</option>
            <option value="4h">4小时</option>
            <option value="1d">1天</option>
          </select>
          <select
            value={timeRange}
            onChange={(e) => setTimeRange(e.target.value)}
            className="border rounded px-2 py-1"
          >
            <option value="1d">1天</option>
            <option value="1w">1周</option>
            <option value="1m">1月</option>
          </select>
        </div>
      </div>

      <div className="h-96">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={klines}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis 
              dataKey="timestamp" 
              tickFormatter={(time) => new Date(time).toLocaleTimeString()}
            />
            <YAxis 
              domain={['auto', 'auto']}
              tickFormatter={(value) => `$${value}`}
            />
            <Tooltip
              labelFormatter={(time) => new Date(time).toLocaleString()}
              formatter={(value) => [`$${value}`, '价格']}
            />
            <Legend />
            <Line 
              type="monotone" 
              dataKey="close" 
              name="收盘价" 
              stroke="#8884d8" 
            />
            <Line 
              type="monotone" 
              dataKey="high" 
              name="最高价" 
              stroke="#82ca9d"
              strokeDasharray="5 5"
            />
            <Line 
              type="monotone" 
              dataKey="low" 
              name="最低价" 
              stroke="#ff7300"
              strokeDasharray="5 5"
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

export default KlineChart;