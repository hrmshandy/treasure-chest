export interface ApiUsage {
  hourlyLimit: number | null;
  hourlyRemaining: number | null;
  hourlyReset: string | null;
  dailyLimit: number | null;
  dailyRemaining: number | null;
  dailyReset: string | null;
  lastUpdated: string | null;
}
