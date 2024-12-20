// src/App.jsx
import { useState } from 'react';
import KlineChart from './components/KlineChart';
import OrderBook from './components/OrderBook';
import TickerCard from './components/TickerCard';
import TradesTable from './components/TradesTable';

function App() {
  const [symbol, setSymbol] = useState('BTCUSDT');

  return (
    <div className="min-h-screen bg-gray-50">
      <header className="bg-white border-b p-4">
        <div className="container mx-auto">
          <div className="flex justify-between items-center">
            <h1 className="text-2xl font-bold">Rust Trade</h1>
            <select 
              value={symbol}
              onChange={(e) => setSymbol(e.target.value)}
              className="p-2 border rounded"
            >
              <option value="BTCUSDT">BTC/USDT</option>
              <option value="ETHUSDT">ETH/USDT</option>
            </select>
          </div>
        </div>
      </header>

      <main className="container mx-auto p-6">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
          <TickerCard symbol={symbol} />
          <OrderBook symbol={symbol} />
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="md:col-span-2">
            <KlineChart symbol={symbol} />
          </div>
          <div>
            <TradesTable symbol={symbol} />
          </div>
        </div>
      </main>
    </div>
  );
}

export default App;