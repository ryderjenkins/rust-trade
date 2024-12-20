import { useState, useEffect } from "react";

function OrderBook({ symbol }) {
  const [orderBook, setOrderBook] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchOrderBook = async () => {
      try {
        const response = await fetch(
          `http://localhost:8080/api/v1/market/orderbook/${symbol}?limit=10`
        );
        const data = await response.json();
        if (data.success) {
          setOrderBook(data.data);
        }
      } catch (error) {
        console.error('Error fetching order book:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchOrderBook();
    const interval = setInterval(fetchOrderBook, 1000);
    return () => clearInterval(interval);
  }, [symbol]);

  if (loading) return <div>加载中...</div>;

  return orderBook && (
    <div className="p-4 border rounded-lg bg-white shadow">
      <h2 className="text-xl font-bold mb-4">订单簿</h2>
      <div className="grid grid-cols-2 gap-4">
        <div>
          <h3 className="text-green-600 font-bold mb-2">买盘</h3>
          {orderBook.bids.map(([price, amount], i) => (
            <div key={i} className="grid grid-cols-2 text-sm mb-1">
              <span className="text-green-600">{price}</span>
              <span>{amount}</span>
            </div>
          ))}
        </div>
        <div>
          <h3 className="text-red-600 font-bold mb-2">卖盘</h3>
          {orderBook.asks.map(([price, amount], i) => (
            <div key={i} className="grid grid-cols-2 text-sm mb-1">
              <span className="text-red-600">{price}</span>
              <span>{amount}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

export default OrderBook;