import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { ApiUsage } from '../../types/apiUsage';
import { Activity } from 'lucide-react';

export const Footer = () => {
  const [apiUsage, setApiUsage] = useState<ApiUsage | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const fetchApiUsage = async () => {
    try {
      const usage = await invoke<ApiUsage>('get_api_usage');
      setApiUsage(usage);
      setIsLoading(false);
    } catch (error) {
      console.error('Failed to fetch API usage:', error);
      setIsLoading(false);
    }
  };

  useEffect(() => {
    // Fetch immediately
    fetchApiUsage();

    // Refresh every 10 seconds
    const interval = setInterval(fetchApiUsage, 10000);

    return () => clearInterval(interval);
  }, []);

  // Show footer even if no data yet
  const hasData = apiUsage && (apiUsage.hourlyLimit || apiUsage.dailyLimit);

  const getUsageColor = (remaining: number | null, limit: number | null) => {
    if (!remaining || !limit) return 'text-stone-500';
    const percentage = (remaining / limit) * 100;
    if (percentage > 50) return 'text-green-400';
    if (percentage > 20) return 'text-orange-400';
    return 'text-red-400';
  };

  const getUsageBarWidth = (remaining: number | null, limit: number | null) => {
    if (!remaining || !limit) return '0%';
    return `${(remaining / limit) * 100}%`;
  };

  return (
    <footer className="border-t border-stone-800 bg-stone-950 px-4 py-2">
      <div className="flex items-center justify-between text-xs">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
            <Activity className="w-3 h-3 text-stone-500" />
            <span className="text-stone-500 font-medium">Nexus API Usage</span>
          </div>

          {!hasData && (
            <span className="text-stone-600 italic">No API calls yet - download a mod to see usage</span>
          )}

          {hasData && apiUsage.hourlyLimit && apiUsage.hourlyRemaining !== null && (
            <div className="flex items-center gap-2">
              <span className="text-stone-600">Hourly:</span>
              <div className="flex items-center gap-1">
                <span className={`font-mono font-medium ${getUsageColor(apiUsage.hourlyRemaining, apiUsage.hourlyLimit)}`}>
                  {apiUsage.hourlyRemaining}
                </span>
                <span className="text-stone-600">/</span>
                <span className="text-stone-500 font-mono">{apiUsage.hourlyLimit}</span>
              </div>
              <div className="w-20 h-1.5 bg-stone-800 rounded-full overflow-hidden">
                <div
                  className={`h-full transition-all duration-300 ${apiUsage.hourlyRemaining / apiUsage.hourlyLimit > 0.5
                    ? 'bg-green-500'
                    : apiUsage.hourlyRemaining / apiUsage.hourlyLimit > 0.2
                      ? 'bg-orange-500'
                      : 'bg-red-500'
                    }`}
                  style={{ width: getUsageBarWidth(apiUsage.hourlyRemaining, apiUsage.hourlyLimit) }}
                />
              </div>
            </div>
          )}

          {hasData && apiUsage.dailyLimit && apiUsage.dailyRemaining !== null && (
            <div className="flex items-center gap-2">
              <span className="text-stone-600">Daily:</span>
              <div className="flex items-center gap-1">
                <span className={`font-mono font-medium ${getUsageColor(apiUsage.dailyRemaining, apiUsage.dailyLimit)}`}>
                  {apiUsage.dailyRemaining}
                </span>
                <span className="text-stone-600">/</span>
                <span className="text-stone-500 font-mono">{apiUsage.dailyLimit}</span>
              </div>
              <div className="w-20 h-1.5 bg-stone-800 rounded-full overflow-hidden">
                <div
                  className={`h-full transition-all duration-300 ${apiUsage.dailyRemaining / apiUsage.dailyLimit > 0.5
                    ? 'bg-green-500'
                    : apiUsage.dailyRemaining / apiUsage.dailyLimit > 0.2
                      ? 'bg-orange-500'
                      : 'bg-red-500'
                    }`}
                  style={{ width: getUsageBarWidth(apiUsage.dailyRemaining, apiUsage.dailyLimit) }}
                />
              </div>
            </div>
          )}
        </div>

        {hasData && apiUsage.lastUpdated && (
          <div className="text-stone-600">
            Last updated: {new Date(apiUsage.lastUpdated).toLocaleTimeString()}
          </div>
        )}
      </div>
    </footer>
  );
};
