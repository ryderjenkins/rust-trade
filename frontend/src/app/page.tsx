// src/app/page.tsx
'use client';

import { useEffect, useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { Loader2 } from 'lucide-react';

interface StrategyNFT {
  id: string;
  name: string;
  rarity: 'Legendary' | 'Epic' | 'Rare' | 'Common';
  metrics: {
    returnRate: number;
    sharpeRatio: number;
    winRate: number;
    maxDrawdown: number;
  };
  parameters: Record<string, string | number>;
  createdAt: string;
  performanceHistory: Array<{ timestamp: string; value: number }>;
}

export default function Home() {
  const [loading, setLoading] = useState(true);
  const [strategyNFTs] = useState<StrategyNFT[]>([
    {
      id: '1',
      name: 'Golden Cross Strategy',
      rarity: 'Legendary',
      metrics: {
        returnRate: 156.7,
        sharpeRatio: 3.45,
        winRate: 68.5,
        maxDrawdown: 15.2,
      },
      parameters: {
        shortPeriod: 5,
        longPeriod: 20,
        positionSize: '15%',
      },
      createdAt: '2024-01-10',
      performanceHistory: Array(30).fill(0).map((_, i) => ({
        timestamp: `2024-01-${i + 1}`,
        value: 10000 * (1 + Math.sin(i / 5) * 0.1 + i / 30),
      })),
    },
    // ... 可以添加更多示例 NFT
  ]);

  // 获取稀有度对应的样式
  const getRarityStyle = (rarity: string) => {
    switch (rarity) {
      case 'Legendary':
        return 'from-yellow-300 to-yellow-600 text-yellow-800 dark:text-yellow-400';
      case 'Epic':
        return 'from-purple-300 to-purple-600 text-purple-800 dark:text-purple-400';
      case 'Rare':
        return 'from-blue-300 to-blue-600 text-blue-800 dark:text-blue-400';
      default:
        return 'from-gray-300 to-gray-600 text-gray-800 dark:text-gray-400';
    }
  };

  useEffect(() => {
    // 模拟加载数据
    setTimeout(() => setLoading(false), 1000);
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  return (
    <div className="container mx-auto p-6">
      <h1 className="text-3xl font-bold mb-8">Strategy NFT Gallery</h1>
      
      {/* NFT Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {strategyNFTs.map((nft) => (
          <div
            key={nft.id}
            className={`bg-gradient-to-br ${getRarityStyle(nft.rarity)} rounded-lg shadow-lg p-6 transform transition-all hover:scale-105`}
          >
            <div className="bg-white/90 dark:bg-gray-800/90 rounded-lg p-4">
              <h3 className="text-xl font-bold">{nft.name}</h3>
              <div className="mt-4 h-40">
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={nft.performanceHistory}>
                    <Line 
                      type="monotone" 
                      dataKey="value" 
                      stroke="#8884d8" 
                      dot={false}
                    />
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="timestamp" hide />
                    <YAxis hide />
                    <Tooltip />
                  </LineChart>
                </ResponsiveContainer>
              </div>
              <div className="mt-4 space-y-2">
                <p className="text-sm">Return Rate: {nft.metrics.returnRate}%</p>
                <p className="text-sm">Sharpe Ratio: {nft.metrics.sharpeRatio}</p>
                <p className="text-sm">Win Rate: {nft.metrics.winRate}%</p>
                <p className="text-sm">Max Drawdown: {nft.metrics.maxDrawdown}%</p>
              </div>
              <div className="mt-4 text-xs text-gray-600">
                <p>Parameters:</p>
                {Object.entries(nft.parameters).map(([key, value]) => (
                  <span key={key} className="mr-2">
                    {key}: {value}
                  </span>
                ))}
              </div>
              <div className="mt-4 text-xs text-gray-500">
                Created: {nft.createdAt}
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Summary Stats */}
      <Card className="mt-8">
        <CardHeader>
          <CardTitle>Top Performing Strategies</CardTitle>
        </CardHeader>
        <CardContent>
          <table className="w-full">
            <thead>
              <tr className="text-left border-b">
                <th className="pb-2">Rank</th>
                <th className="pb-2">Strategy</th>
                <th className="pb-2">Return</th>
                <th className="pb-2">Sharpe</th>
                <th className="pb-2">Rarity</th>
              </tr>
            </thead>
            <tbody>
              {strategyNFTs
                .sort((a, b) => b.metrics.returnRate - a.metrics.returnRate)
                .map((nft, index) => (
                  <tr key={nft.id} className="border-b">
                    <td className="py-2">#{index + 1}</td>
                    <td className="py-2">{nft.name}</td>
                    <td className="py-2 text-green-500">+{nft.metrics.returnRate}%</td>
                    <td className="py-2">{nft.metrics.sharpeRatio}</td>
                    <td className={`py-2 ${getRarityStyle(nft.rarity)}`}>{nft.rarity}</td>
                  </tr>
                ))}
            </tbody>
          </table>
        </CardContent>
      </Card>
    </div>
  );
}