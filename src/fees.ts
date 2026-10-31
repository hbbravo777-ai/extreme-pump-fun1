/**
 * Fee accounting. 10,000 basis points = 100%.
 * Platform and creator fees are separate charges on gross input.
 */
export const BPS_DENOMINATOR = 10_000n;
export const PLATFORM_FEE_BPS = 100n;

export interface FeeConfig {
  platformFeeBps: bigint;
  creatorFeeBps: bigint;
}
export interface FeeBreakdown {
  grossAmount: bigint;
  platformFee: bigint;
  creatorFee: bigint;
  netAmount: bigint;
  totalFee: bigint;
}
function assertAmount(amount: bigint): void {
  if (amount < 0n) throw new RangeError("Amount cannot be negative");
}
function assertBps(rate: bigint, label: string): void {
  if (rate < 0n || rate >= BPS_DENOMINATOR) {
    throw new RangeError(label + " must be between 0 and 9,999 bps");
  }
}
export function calculateFees(grossAmount: bigint, config: FeeConfig): FeeBreakdown {
  assertAmount(grossAmount);
  assertBps(config.platformFeeBps, "Platform fee");
  assertBps(config.creatorFeeBps, "Creator fee");
  const platformFee = grossAmount * config.platformFeeBps / BPS_DENOMINATOR;
  const creatorFee = grossAmount * config.creatorFeeBps / BPS_DENOMINATOR;
  const totalFee = platformFee + creatorFee;
  if (totalFee > grossAmount) throw new RangeError("Combined fees cannot exceed gross amount");
  return { grossAmount, platformFee, creatorFee, netAmount: grossAmount - totalFee, totalFee };
}
