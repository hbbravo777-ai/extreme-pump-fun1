import "dotenv/config";
import fs from "node:fs";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import {
  Raydium,
  TxVersion,
  DEVNET_PROGRAM_ID,
  getCpmmPdaAmmConfigId,
} from "@raydium-io/raydium-sdk-v2";

const required = (key: string): string => {
  const value = process.env[key];
  if (!value) throw new Error(`Missing required environment variable: ${key}`);
  return value;
};

async function main(): Promise<void> {
  if (process.env.ALLOW_RAYDIUM_POOL_CREATION !== "YES") {
    throw new Error("Refusing to sign. Set ALLOW_RAYDIUM_POOL_CREATION=YES only after reviewing the pool parameters.");
  }
  const rpcUrl = process.env.RPC_URL ?? "https://api.devnet.solana.com";
  const keypairPath = required("KEYPAIR_PATH");
  const tokenMint = new PublicKey(required("TOKEN_MINT"));
  const tokenAmount = required("TOKEN_AMOUNT_BASE_UNITS");
  const solAmount = required("SOL_AMOUNT_LAMPORTS");
  const feeConfigIndex = Number(process.env.CPMM_FEE_CONFIG_INDEX ?? "0");
  if (!/^\d+$/.test(tokenAmount) || !/^\d+$/.test(solAmount)) throw new Error("Amounts must be unsigned integer base units.");
  if (BigInt(tokenAmount) <= 0n || BigInt(solAmount) <= 0n) throw new Error("Both liquidity amounts must be positive.");
  if (!Number.isSafeInteger(feeConfigIndex) || feeConfigIndex < 0) throw new Error("Invalid CPMM_FEE_CONFIG_INDEX.");
  const secret = JSON.parse(fs.readFileSync(keypairPath, "utf8")) as number[];
  const owner = Keypair.fromSecretKey(Uint8Array.from(secret));
  const connection = new Connection(rpcUrl, "confirmed");
  const raydium = await Raydium.load({
    connection,
    owner,
    cluster: "devnet",
    disableFeatureCheck: true,
    disableLoadToken: false,
  });
  if (raydium.cluster !== "devnet") throw new Error("This script is devnet-only.");
  const mintA = await raydium.token.getTokenInfo(tokenMint.toBase58());
  const wrappedSolMint = new PublicKey("So11111111111111111111111111111111111111112");
  const mintB = await raydium.token.getTokenInfo(wrappedSolMint.toBase58());
  const configs = await raydium.api.getCpmmConfigs();
  if (!configs[feeConfigIndex]) throw new Error("Requested CPMM fee config index does not exist.");
  const feeConfig = {
    ...configs[feeConfigIndex],
    id: getCpmmPdaAmmConfigId(DEVNET_PROGRAM_ID.CREATE_CPMM_POOL_PROGRAM, configs[feeConfigIndex].index).publicKey.toBase58(),
  };

  console.log("DEVNET CPMM pool review");
  console.log({ owner: owner.publicKey.toBase58(), mintA: tokenMint.toBase58(), mintB: wrappedSolMint.toBase58(), tokenAmount, solAmount, feeConfig: feeConfig.id });
  console.log("The wallet must already hold both assets. This script does not withdraw curve vault funds or prove graduation eligibility.");

  const { execute, extInfo } = await raydium.cpmm.createPool({
    programId: DEVNET_PROGRAM_ID.CREATE_CPMM_POOL_PROGRAM,
    poolFeeAccount: DEVNET_PROGRAM_ID.CREATE_CPMM_POOL_FEE_ACC,
    mintA,
    mintB,
    mintAAmount: new BN(tokenAmount),
    mintBAmount: new BN(solAmount),
    startTime: new BN(0),
    feeConfig,
    associatedOnly: false,
    ownerInfo: { useSOLBalance: true },
    txVersion: TxVersion.V0,
  });
  const { txId } = await execute({ sendAndConfirm: true });
  console.log("CPMM pool transaction confirmed:", txId);
  console.log("Pool address:", extInfo.address.poolId.toBase58());
  console.log("Explorer:", `https://explorer.solana.com/tx/${txId}?cluster=devnet`);
}

main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
