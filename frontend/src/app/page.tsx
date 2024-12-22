// src/app/page.tsx
export default function Home() {
  return (
    <div className="grid gap-6">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {/* Market Overview Card */}
        <div className="p-6 bg-white rounded-lg shadow">
          <h2 className="text-xl font-semibold mb-4">Market Overview</h2>
          <div className="space-y-2">
            <p>BTC/USDT: $45,000</p>
            <p>24h Volume: $1.2B</p>
          </div>
        </div>
        
        {/* Trading Stats Card */}
        <div className="p-6 bg-white rounded-lg shadow">
          <h2 className="text-xl font-semibold mb-4">Trading Stats</h2>
          <div className="space-y-2">
            <p>Total Trades: 150</p>
            <p>Win Rate: 65%</p>
          </div>
        </div>
        
        {/* Portfolio Card */}
        <div className="p-6 bg-white rounded-lg shadow">
          <h2 className="text-xl font-semibold mb-4">Portfolio</h2>
          <div className="space-y-2">
            <p>Total Value: $100,000</p>
            <p>Daily P&L: +2.5%</p>
          </div>
        </div>
      </div>
    </div>
  )
}