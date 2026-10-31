import { useMemo, useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { WalletMultiButton } from "@solana/wallet-adapter-react-ui";

const configuredProgram = Boolean(import.meta.env.VITE_PROGRAM_ID && import.meta.env.VITE_PROGRAM_ID !== "11111111111111111111111111111111");
const cluster = import.meta.env.VITE_SOLANA_CLUSTER || "devnet";

function shortAddress(value: string) {
  return value.length > 12 ? value.slice(0, 5) + "…" + value.slice(-5) : value;
}

export default function App() {
  const { publicKey, connected } = useWallet();
  const [query, setQuery] = useState("");
  const [tab, setTab] = useState<"trending" | "new">("trending");
  const walletLabel = useMemo(() => publicKey ? shortAddress(publicKey.toBase58()) : "Connect wallet", [publicKey]);

  return (
    <main className="shell">
      <header className="topbar">
        <a className="brand" href="#"><span className="brand-mark">X</span><span>EXTREME <b>PUMP.FUN</b><small>SOLANA TOKEN LAUNCHPAD</small></span></a>
        <nav><a href="#explore">Explore</a><a href="#how">How it works</a><a href="https://raydium.io/" target="_blank" rel="noreferrer">Raydium ↗</a></nav>
        <div className="wallet"><span className="network"><i />{cluster}</span><WalletMultiButton>{walletLabel}</WalletMultiButton></div>
      </header>

      <section className="hero">
        <div className="eyebrow"><span className="pulse" /> THE NEXT WAVE STARTS HERE</div>
        <h1>MAKE A TOKEN.<br /><em>MAKE SOME NOISE.</em></h1>
        <p>Launch community tokens on Solana. Discover new ideas, follow the curve, and trade with transparent on-chain fees.</p>
        <div className="hero-actions"><a className="primary" href="#launch">Launch a token <span>↗</span></a><a className="secondary" href="#explore">Explore tokens</a></div>
        <div className="hero-art" aria-hidden="true"><div className="orbit orbit-one" /><div className="orbit orbit-two" /><div className="coin">X</div><div className="spark spark-a">✳</div><div className="spark spark-b">✦</div></div>
      </section>

      <section className="status-row">
        <div><span className="status-dot" /> NETWORK STATUS <strong>{cluster.toUpperCase()}</strong></div>
        <div>PLATFORM FEE <strong>1.00%</strong></div>
        <div>CREATOR FEE <strong>SET PER TOKEN</strong></div>
        <div>TRADING <strong className={configuredProgram ? "yellow" : "muted"}>{configuredProgram ? "PROGRAM CONFIGURED" : "NOT ENABLED"}</strong></div>
      </section>

      <section id="launch" className="launch-card">
        <div><div className="eyebrow">YOUR NEXT COMMUNITY TOKEN</div><h2>Ready to launch?</h2><p>Connect your wallet to prepare a token launch. Token creation and on-chain trading remain disabled until the program ID, deployed IDL, and audited launch flow are configured.</p></div>
        <div className="launch-side"><div className="launch-icon">＋</div><button disabled={!connected || !configuredProgram}>Create token <span>↗</span></button><small>{!connected ? "Connect a Solana wallet to continue" : !configuredProgram ? "Awaiting verified program deployment" : "Launch flow awaiting integration"}</small></div>
      </section>

      <section id="explore" className="explore">
        <div className="section-heading"><div><div className="eyebrow">THE COMMUNITY FEED</div><h2>Explore tokens</h2></div><div className="feed-tools"><div className="tabs"><button className={tab === "trending" ? "active" : ""} onClick={() => setTab("trending")}>Trending</button><button className={tab === "new" ? "active" : ""} onClick={() => setTab("new")}>New</button></div><input aria-label="Search tokens" placeholder="Search tokens…" value={query} onChange={e => setQuery(e.target.value)} /></div></div>
        <div className="empty-feed"><div className="empty-glyph">⌁</div><h3>{query ? "No indexed matches yet" : tab === "new" ? "No new launches indexed" : "The feed is warming up"}</h3><p>Token discovery appears here when the indexer is connected to the deployed program.</p></div>
      </section>

      <section id="how" className="how">
        <div className="eyebrow">BUILT FOR TRANSPARENCY</div><h2>Simple by design.<br /><em>On-chain by default.</em></h2>
        <div className="feature-grid"><article><span>01</span><h3>Launch</h3><p>Create a token and seed its bonding curve with an initial inventory.</p></article><article><span>02</span><h3>Trade</h3><p>Buy and sell against transparent curve pricing with minimum-output protection.</p></article><article><span>03</span><h3>Graduate</h3><p>Eligible curves can migrate liquidity to a Raydium pool after the migration flow is audited.</p></article></div>
      </section>

      <footer><a className="brand" href="#"><span className="brand-mark">X</span><span>EXTREME <b>PUMP.FUN</b><small>BUILT ON SOLANA</small></span></a><span>Independent community launchpad. Not affiliated with other launch platforms.</span><span>{connected && publicKey ? walletLabel : "Wallet not connected"}</span></footer>
    </main>
  );
}
