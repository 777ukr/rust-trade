// src/app/backtest/page.tsx
'use client';

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  BacktestRequest,
  BacktestResponse,
  DataInfoResponse,
  StrategyInfo
} from '@/types/backtest';
import { AlertCircle, CheckCircle, Database, Loader2, TrendingUp } from 'lucide-react';
import { useEffect, useState } from 'react';
import { CartesianGrid, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';

// Check if Tauri is available (only in Tauri desktop app)
const isTauriAvailable = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

// API base URL - defaults to localhost:8080 (Rust backend)
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

async function invokeTauri(command: string, args?: any): Promise<any> {
  if (!isTauriAvailable) {
    throw new Error('Tauri API not available. Please use the desktop application.');
  }
  
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke(command, args);
}

// HTTP API client for browser mode
async function fetchAPI<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const url = `${API_BASE_URL}${endpoint}`;
  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    throw new Error(`API request failed: ${response.status} ${response.statusText}`);
  }

  return response.json();
}

interface BacktestParams {
  strategy_id: string;
  symbol: string;
  data_count: number;
  initial_capital: string;
  commission_rate: string;
  short_period: string;
  long_period: string;
  [key: string]: string | number;
}

export default function BacktestPage() {
  // State for data info
  const [dataInfo, setDataInfo] = useState<DataInfoResponse | null>(null);
  const [strategies, setStrategies] = useState<StrategyInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // State for backtest configuration
  const [params, setParams] = useState<BacktestParams>({
    strategy_id: '',
    symbol: '',
    data_count: 10000,
    initial_capital: '10000',
    commission_rate: '0.001',
    short_period: '5',
    long_period: '20',
  });

  // State for validation and execution
  const [configValid, setConfigValid] = useState<boolean | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [result, setResult] = useState<BacktestResponse | null>(null);

  // Initialize data on component mount
  useEffect(() => {
    initializeData();
  }, []);

  // Validate configuration when params change
  useEffect(() => {
    if (params.symbol && params.data_count > 0) {
      validateConfiguration();
    }
  }, [params.symbol, params.data_count]);

  const initializeData = async () => {
    try {
      setLoading(true);
      setError(null);

      // Try HTTP API first (for browser mode), fallback to Tauri
      if (!isTauriAvailable) {
        // Use HTTP API
        console.log('Tauri not available, using HTTP API:', API_BASE_URL);
        
        try {
          const [dataInfoResult, strategiesResult] = await Promise.all([
            fetchAPI<DataInfoResponse>('/api/data/info'),
            fetchAPI<StrategyInfo[]>('/api/strategies')
          ]);

          setDataInfo(dataInfoResult);
          setStrategies(strategiesResult);

          // Set default values
          if (strategiesResult.length > 0) {
            setParams(prev => ({ ...prev, strategy_id: strategiesResult[0].id }));
          }
          if (dataInfoResult.symbol_info.length > 0) {
            setParams(prev => ({ ...prev, symbol: dataInfoResult.symbol_info[0].symbol }));
          }
        } catch (err) {
          console.error('Failed to load data from HTTP API:', err);
          setError(`Failed to connect to API server at ${API_BASE_URL}. Please ensure the backend is running: cargo run api`);
        }
        setLoading(false);
        return;
      }

      // Load data info and strategies in parallel (Tauri mode)
      const [dataInfoResult, strategiesResult] = await Promise.all([
        invokeTauri<DataInfoResponse>('get_data_info'),
        invokeTauri<StrategyInfo[]>('get_available_strategies')
      ]);

      setDataInfo(dataInfoResult);
      setStrategies(strategiesResult);

      // Set default values
      if (strategiesResult.length > 0) {
        setParams(prev => ({ ...prev, strategy_id: strategiesResult[0].id }));
      }
      if (dataInfoResult.symbol_info.length > 0) {
        setParams(prev => ({ ...prev, symbol: dataInfoResult.symbol_info[0].symbol }));
      }

    } catch (err) {
      console.error('Failed to initialize data:', err);
      setError(err instanceof Error ? err.message : 'Failed to load data');
    } finally {
      setLoading(false);
    }
  };

  const validateConfiguration = async () => {
    try {
      if (!isTauriAvailable) {
        // Use HTTP API
        try {
          const isValid = await fetchAPI<boolean>(
            `/api/backtest/validate?symbol=${encodeURIComponent(params.symbol)}&data_count=${params.data_count}`
          );
          setConfigValid(isValid);
        } catch (err) {
          console.error('Validation failed:', err);
          // Fallback: just check if basic params are set
          setConfigValid(params.symbol.length > 0 && params.data_count > 0);
        }
        return;
      }

      const isValid = await invokeTauri<boolean>('validate_backtest_config', {
        symbol: params.symbol,
        dataCount: params.data_count
      });
      setConfigValid(isValid);
    } catch (err) {
      console.error('Validation failed:', err);
      setConfigValid(false);
    }
  };

  const runBacktest = async () => {
    if (!isTauriAvailable) {
      setError('⚠️ Running backtests via HTTP API is not yet implemented. Please use the desktop application (npx tauri dev) or run backtests via CLI (cargo run backtest)');
      return;
    }

    if (!configValid) {
      setError('Configuration is not valid');
      return;
    }

    try {
      setIsRunning(true);
      setError(null);
      setResult(null);

      const request: BacktestRequest = {
        strategy_id: params.strategy_id,
        symbol: params.symbol,
        data_count: params.data_count,
        initial_capital: params.initial_capital,
        commission_rate: params.commission_rate,
        strategy_params: {
          short_period: params.short_period,
          long_period: params.long_period,
        }
      };

      console.log('Running backtest with request:', request);
      const response = await invokeTauri<BacktestResponse>('run_backtest', { request });
      console.log('Backtest completed:', response);
      setResult(response);

    } catch (err) {
      console.error('Backtest failed:', err);
      setError(err instanceof Error ? err.message : 'Backtest failed');
    } finally {
      setIsRunning(false);
    }
  };

  const formatPercentage = (value: string | number) => {
    const num = typeof value === 'string' ? parseFloat(value) : value;
    return num.toFixed(2);
  };

  const formatPrice = (value: string) => {
    return parseFloat(value).toFixed(2);
  };

  const formatQuantity = (value: string) => {
    return parseFloat(value).toFixed(8);
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader2 className="w-8 h-8 animate-spin" />
        <span className="ml-2">Loading trading data...</span>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      <h1 className="text-3xl font-bold">Strategy Backtesting</h1>

      {/* Data Information Section */}
      {dataInfo && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Database className="w-5 h-5" />
              Database Information
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
              <div>
                <p className="text-sm text-gray-500">Total Records</p>
                <p className="text-2xl font-bold">{dataInfo.total_records.toLocaleString()}</p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Available Symbols</p>
                <p className="text-2xl font-bold">{dataInfo.symbols_count}</p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Earliest Data</p>
                <p className="text-sm font-medium">
                  {dataInfo.earliest_time ? new Date(dataInfo.earliest_time).toLocaleDateString() : 'N/A'}
                </p>
              </div>
              <div>
                <p className="text-sm text-gray-500">Latest Data</p>
                <p className="text-sm font-medium">
                  {dataInfo.latest_time ? new Date(dataInfo.latest_time).toLocaleDateString() : 'N/A'}
                </p>
              </div>
            </div>
            
            <div className="mt-4">
              <p className="text-sm font-medium mb-2">Top Symbols by Records:</p>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-2">
                {dataInfo.symbol_info.slice(0, 6).map((symbol) => (
                  <div key={symbol.symbol} className="text-xs bg-gray-100 dark:bg-gray-800 p-2 rounded">
                    <span className="font-medium">{symbol.symbol}</span>
                    <span className="text-gray-500 ml-2">({symbol.records_count.toLocaleString()} records)</span>
                  </div>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Configuration Section */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <TrendingUp className="w-5 h-5" />
            Backtest Configuration
            {configValid !== null && (
              configValid ? (
                <CheckCircle className="w-5 h-5 text-green-500" />
              ) : (
                <AlertCircle className="w-5 h-5 text-red-500" />
              )
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {/* Strategy Selection */}
            <div>
              <label className="block text-sm font-medium mb-1">Strategy</label>
              <select
                value={params.strategy_id}
                onChange={(e) => setParams({ ...params, strategy_id: e.target.value })}
                className="w-full p-2 border rounded dark:bg-gray-800 dark:border-gray-600"
              >
                <option value="">Select Strategy</option>
                {strategies.map((strategy) => (
                  <option key={strategy.id} value={strategy.id}>
                    {strategy.name}
                  </option>
                ))}
              </select>
              {params.strategy_id && (
                <p className="text-xs text-gray-500 mt-1">
                  {strategies.find(s => s.id === params.strategy_id)?.description}
                </p>
              )}
            </div>

            {/* Symbol Selection */}
            <div>
              <label className="block text-sm font-medium mb-1">Symbol</label>
              <select
                value={params.symbol}
                onChange={(e) => setParams({ ...params, symbol: e.target.value })}
                className="w-full p-2 border rounded dark:bg-gray-800 dark:border-gray-600"
              >
                <option value="">Select Symbol</option>
                {dataInfo?.symbol_info.map((symbol) => (
                  <option key={symbol.symbol} value={symbol.symbol}>
                    {symbol.symbol} ({symbol.records_count.toLocaleString()} records)
                  </option>
                ))}
              </select>
            </div>

            {/* Data Count */}
            <div>
              <label className="block text-sm font-medium mb-1">Data Points</label>
              <input
                type="number"
                value={params.data_count}
                onChange={(e) => setParams({ ...params, data_count: parseInt(e.target.value) || 0 })}
                className="w-full p-2 border rounded dark:bg-gray-800 dark:border-gray-600"
                min="100"
                max="100000"
              />
              {params.symbol && dataInfo && (
                <p className="text-xs text-gray-500 mt-1">
                  Max available: {dataInfo.symbol_info.find(s => s.symbol === params.symbol)?.records_count.toLocaleString() || 0}
                </p>
              )}
            </div>

            {/* Initial Capital */}
            <div>
              <label className="block text-sm font-medium mb-1">Initial Capital ($)</label>
              <input
                type="text"
                value={params.initial_capital}
                onChange={(e) => setParams({ ...params, initial_capital: e.target.value })}
                className="w-full p-2 border rounded dark:bg-gray-800 dark:border-gray-600"
                placeholder="10000"
              />
            </div>

            {/* Commission Rate */}
            <div>
              <label className="block text-sm font-medium mb-1">Commission Rate (%)</label>
              <input
                type="text"
                value={(parseFloat(params.commission_rate) * 100).toString()}
                onChange={(e) => {
                  const percent = parseFloat(e.target.value) || 0;
                  setParams({ ...params, commission_rate: (percent / 100).toString() });
                }}
                className="w-full p-2 border rounded dark:bg-gray-800 dark:border-gray-600"
                placeholder="0.1"
              />
            </div>

            {/* Strategy Parameters */}
            {params.strategy_id === 'sma' && (
              <>
                <div>
                  <label className="block text-sm font-medium mb-1">Short Period</label>
                  <input
                    type="number"
                    value={params.short_period}
                    onChange={(e) => setParams({ ...params, short_period: e.target.value })}
                    className="w-full p-2 border rounded dark:bg-gray-800 dark:border-gray-600"
                    min="1"
                    max="100"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium mb-1">Long Period</label>
                  <input
                    type="number"
                    value={params.long_period}
                    onChange={(e) => setParams({ ...params, long_period: e.target.value })}
                    className="w-full p-2 border rounded dark:bg-gray-800 dark:border-gray-600"
                    min="1"
                    max="200"
                  />
                </div>
              </>
            )}
          </div>

          {/* Validation Status */}
          {configValid !== null && (
            <div className={`mt-4 p-3 rounded ${configValid ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'}`}>
              {configValid ? (
                <p className="flex items-center gap-2">
                  <CheckCircle className="w-4 h-4" />
                  Configuration is valid and ready for backtesting
                </p>
              ) : (
                <p className="flex items-center gap-2">
                  <AlertCircle className="w-4 h-4" />
                  Insufficient data for the selected configuration
                </p>
              )}
            </div>
          )}

          {/* Run Button */}
          <button
            onClick={runBacktest}
            disabled={!configValid || isRunning || !params.strategy_id || !params.symbol}
            className={`mt-4 px-6 py-2 rounded font-medium ${
              configValid && !isRunning && params.strategy_id && params.symbol
                ? 'bg-blue-500 hover:bg-blue-600 text-white'
                : 'bg-gray-400 text-gray-600 cursor-not-allowed'
            }`}
          >
            {isRunning ? (
              <span className="flex items-center gap-2">
                <Loader2 className="w-4 h-4 animate-spin" />
                Running Backtest...
              </span>
            ) : (
              'Run Backtest'
            )}
          </button>
        </CardContent>
      </Card>

      {/* Info/Warning Display */}
      {!isTauriAvailable && (
        <Card className="border-blue-200 bg-blue-50 dark:border-blue-800 dark:bg-blue-900/20">
          <CardContent className="pt-6">
            <div className="flex items-center gap-2 text-blue-800 dark:text-blue-200">
              <AlertCircle className="w-5 h-5" />
              <div>
                <span className="font-medium">Browser Mode:</span>
                <span className="ml-2">Using HTTP API at {API_BASE_URL}. Make sure the backend is running:</span>
                <code className="ml-2 px-2 py-1 bg-blue-100 dark:bg-blue-900 rounded text-sm">cd trading-core && cargo run api</code>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Error Display */}
      {error && (
        <Card className="border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-900/20">
          <CardContent className="pt-6">
            <div className="flex items-center gap-2 text-red-800 dark:text-red-200">
              <AlertCircle className="w-5 h-5" />
              <span className="font-medium">Error:</span>
              <span>{error}</span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Results Section */}
      {result && (
        <div className="space-y-6">
          {/* Summary Metrics */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center justify-between">
                <span>Backtest Results - {result.strategy_name}</span>
                <span className={`text-sm px-3 py-1 rounded-full ${
                  result.data_source.startsWith('OHLC') 
                    ? 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200'
                    : 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200'
                }`}>
                  {result.data_source.startsWith('OHLC') 
                    ? `${result.data_source} K-line Data` 
                    : 'Tick Data'
                  }
                </span>
              </CardTitle>
            </CardHeader>
            <CardContent>
              {result.data_source.startsWith('OHLC') && (
                <div className="mb-4 p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                  <p className="text-sm text-blue-800 dark:text-blue-200">
                    📈 This backtest used {result.data_source} candlestick data for improved performance and reduced noise.
                  </p>
                </div>
              )}
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4">
                <div>
                  <p className="text-sm text-gray-500">Total Return</p>
                  <p className={`text-xl font-bold ${parseFloat(result.return_percentage) >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                    {formatPercentage(result.return_percentage)}%
                  </p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Final Value</p>
                  <p className="text-xl font-bold">${formatPrice(result.final_value)}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Total P&L</p>
                  <p className={`text-xl font-bold ${parseFloat(result.total_pnl) >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                    ${formatPrice(result.total_pnl)}
                  </p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Sharpe Ratio</p>
                  <p className="text-xl font-bold">{formatPercentage(result.sharpe_ratio)}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Max Drawdown</p>
                  <p className="text-xl font-bold text-red-500">{formatPercentage(result.max_drawdown)}%</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Win Rate</p>
                  <p className="text-xl font-bold">{formatPercentage(result.win_rate)}%</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Total Trades</p>
                  <p className="text-xl font-bold">{result.total_trades}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Winning Trades</p>
                  <p className="text-xl font-bold text-green-500">{result.winning_trades}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Losing Trades</p>
                  <p className="text-xl font-bold text-red-500">{result.losing_trades}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Profit Factor</p>
                  <p className="text-xl font-bold">{formatPercentage(result.profit_factor)}</p>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Equity Curve */}
          {result.equity_curve && result.equity_curve.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Portfolio Equity Curve</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="h-96">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart
                      data={result.equity_curve.map((value, index) => ({
                        index,
                        value: parseFloat(value),
                      }))}
                    >
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis 
                        dataKey="index"
                        tickFormatter={(value) => `${value}`}
                      />
                      <YAxis
                        domain={['auto', 'auto']}
                        tickFormatter={(value) => `$${value.toFixed(0)}`}
                      />
                      <Tooltip
                        formatter={(value: number) => [`$${value.toFixed(2)}`, 'Portfolio Value']}
                        labelFormatter={(index) => `Trade #${index}`}
                      />
                      <Line
                        type="monotone"
                        dataKey="value"
                        stroke="#2563eb"
                        dot={false}
                        strokeWidth={2}
                      />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Trade History */}
          {result.trades && result.trades.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Trade History ({result.trades.length} trades)</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="overflow-x-auto">
                  <table className="w-full">
                    <thead>
                      <tr className="text-left border-b">
                        <th className="pb-2">#</th>
                        <th className="pb-2">Time</th>
                        <th className="pb-2">Side</th>
                        <th className="pb-2">Symbol</th>
                        <th className="pb-2">Quantity</th>
                        <th className="pb-2">Price</th>
                        <th className="pb-2">P&L</th>
                        <th className="pb-2">Commission</th>
                      </tr>
                    </thead>
                    <tbody>
                      {result.trades.slice(0, 50).map((trade, index) => (
                        <tr key={index} className="border-b">
                          <td className="py-2">{index + 1}</td>
                          <td className="py-2">{new Date(trade.timestamp).toLocaleString()}</td>
                          <td className={`py-2 font-medium ${trade.side === 'Buy' ? 'text-green-500' : 'text-red-500'}`}>
                            {trade.side}
                          </td>
                          <td className="py-2">{trade.symbol}</td>
                          <td className="py-2">{formatQuantity(trade.quantity)}</td>
                          <td className="py-2">${formatPrice(trade.price)}</td>
                          <td className={`py-2 font-medium ${
                            trade.realized_pnl 
                              ? parseFloat(trade.realized_pnl) >= 0 ? 'text-green-500' : 'text-red-500'
                              : ''
                          }`}>
                            {trade.realized_pnl ? `$${formatPrice(trade.realized_pnl)}` : '-'}
                          </td>
                          <td className="py-2">${formatPrice(trade.commission)}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                  {result.trades.length > 50 && (
                    <p className="text-sm text-gray-500 mt-2">
                      Showing first 50 trades of {result.trades.length} total trades
                    </p>
                  )}
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}
    </div>
  );
}