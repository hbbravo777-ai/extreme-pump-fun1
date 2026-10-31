/**
 * Constant-product quote helper for local simulation and UI previews.
 * This is NOT an on-chain bonding-curve implementation.
 */
import { calculateFees, type FeeConfig } from "./fees.js";
export interface CurveReserves {
  virtualSolLamports: bigint;
  virtualTokenUnits: bigint;
}
export interface BuyQuote {
  inputSolLamports: bigint;
  fees: ReturnType<typeof calculateFees>;
  tokenOut: bigint;
  nextReserves: CurveReserves;
  priceImpactBps: bigint;
}
export function quoteBuy(inputSolLamports: bigint, reserves: CurveReserves, feeConfig: FeeConfig): BuyQuote {
  if (inputSolLamports <= 0n) throw new RangeError("Input must be positive");
  if (reserves.virtualSolLamports <= 0n || reserves.virtualTokenUnits <= 0n) {
    throw new RangeError("Virtual reserves must be positive");
  }
  const fees = calculateFees(inputSolLamports, feeConfig);
  if (fees.netAmount <= 0n) throw new RangeError("Net input must be positive");
  const invariant = reserves.virtualSolLamports * reserves.virtualTokenUnits;
  const newSolReserve = reserves.virtualSolLamports + fees.netAmount;
  const newTokenReserve = (invariant + newSolReserve - 1n) / newSolReserve;
  const tokenOut = reserves.virtualTokenUnits - newTokenReserve;
  if (tokenOut <= 0n) throw new RangeError("Trade is too small to return one token unit");
  const idealOut = reserves.virtualTokenUnits * fees.netAmount / reserves.virtualSolLamports;
  const priceImpactBps = idealOut > 0n ? (idealOut - tokenOut) * 10_000n / idealOut : 0n;
  return {
    inputSolLamports, fees, tokenOut,
    nextReserves: { virtualSolLamports: newSolReserve, virtualTokenUnits: newTokenReserve },
    priceImpactBps
  };
}
