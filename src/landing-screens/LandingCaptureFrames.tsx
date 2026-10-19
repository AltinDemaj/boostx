import type React from "react";
import "../index.css";

function Frame({ id, children }: { id: string; children: React.ReactNode }) {
  return (
    <div id={id} className="w-[1100px] min-h-[680px] bg-[#07090f] p-6 box-border">
      {children}
    </div>
  );
}

function Glass({ children, className = "" }: { children: React.ReactNode; className?: string }) {
  return <div className={`glass rounded-xl p-4 ${className}`}>{children}</div>;
}

export function LandingCaptureFrames() {
  return (
    <div className="space-y-8 bg-[#07090f]">
      <Frame id="shot-dashboard">
        <header className="border-b border-white/10 pb-4 mb-6">
          <p className="text-xs uppercase tracking-[0.2em] text-teal-300/80">BoostX</p>
          <h1 className="text-xl font-semibold text-white">Competitive PC optimizer</h1>
        </header>
        <section className="hero-auto mb-6">
          <h2 className="text-2xl font-bold text-white">Auto Optimize</h2>
          <p className="text-sm text-slate-400 mt-2">Baseline → A/B tweak tests → keep winners → report card</p>
          <button type="button" className="btn-hero mt-4 px-8 py-3 rounded-xl text-base font-bold">
            Auto Optimize
          </button>
        </section>
        <div className="flex gap-6 items-center">
          <div
            className="score-ring rounded-full w-44 h-44 flex items-center justify-center"
            style={{ "--score": 78 } as React.CSSProperties}
          >
            <div className="w-36 h-36 rounded-full bg-[#0d111c] flex flex-col items-center justify-center border border-white/10">
              <span className="text-3xl font-bold text-white">78</span>
              <span className="text-xs text-slate-400">Score</span>
            </div>
          </div>
          <Glass className="flex-1">
            <h3 className="text-white font-medium">Quick actions</h3>
            <div className="flex gap-2 mt-3">
              <span className="px-4 py-2 rounded-lg bg-teal-500 text-slate-900 text-sm font-semibold">Competitive</span>
              <span className="px-4 py-2 rounded-lg bg-indigo-500 text-white text-sm">Balanced</span>
            </div>
          </Glass>
        </div>
      </Frame>

      <Frame id="shot-optimize">
        <h2 className="text-lg font-semibold text-white mb-4">Tweak catalog</h2>
        <div className="space-y-3">
          {[
            ["Minimum processor state 100%", "Power & CPU", "High FPS · High latency"],
            ["Disable Xbox Game Bar", "Visuals", "Medium FPS · Medium latency"],
            ["TCP Initial RTO (300 ms)", "Network", "Medium FPS · Medium latency"],
          ].map(([name, cat, impact]) => (
            <Glass key={name} className="flex justify-between items-center gap-4">
              <div>
                <p className="text-white font-medium">{name}</p>
                <p className="text-xs text-slate-500">{cat}</p>
                <p className="impact-meter impact-pill-high mt-1">{impact}</p>
              </div>
              <span className="px-4 py-2 rounded-lg bg-teal-500 text-slate-900 text-sm font-semibold shrink-0">Apply</span>
            </Glass>
          ))}
        </div>
      </Frame>

      <Frame id="shot-benchmark-live">
        <h2 className="text-lg font-semibold text-white mb-2">Live CS2 benchmark</h2>
        <Glass>
          <div className="h-48 flex items-end gap-1 bg-black/30 rounded-lg p-3">
            {Array.from({ length: 48 }).map((_, i) => (
              <div
                key={i}
                className="flex-1 bg-gradient-to-t from-cyan-500 to-violet-400 rounded-sm min-w-[2px]"
                style={{ height: `${40 + (i % 7) * 8}%` }}
              />
            ))}
          </div>
          <p className="text-xs text-slate-500 mt-2">312.4 FPS avg · jitter 0.84 ms · PresentMon</p>
        </Glass>
      </Frame>

      <Frame id="shot-benchmark">
        <h2 className="text-lg font-semibold text-white mb-4">CS2 competitive benchmark</h2>
        <div className="grid grid-cols-2 gap-4">
          <Glass>
            <p className="text-slate-500 text-sm">Baseline</p>
            <p className="text-2xl font-mono text-white mt-1">298.2 avg</p>
            <p className="text-teal-300 font-mono">241.5 1% low</p>
          </Glass>
          <Glass>
            <p className="text-slate-500 text-sm">After tweaks</p>
            <p className="text-2xl font-mono text-white mt-1">312.8 avg</p>
            <p className="text-teal-300 font-mono">256.1 1% low</p>
          </Glass>
        </div>
        <p className="text-sm text-teal-200 mt-4">+14.6 avg FPS vs baseline · 3-run aggregate</p>
      </Frame>

      <Frame id="shot-performance">
        <h2 className="text-lg font-semibold text-white mb-4">Performance</h2>
        <div className="grid grid-cols-2 gap-4">
          <Glass>
            <h3 className="text-white">Timer resolution</h3>
            <p className="text-3xl font-mono text-teal-300 mt-2">0.5 µs</p>
          </Glass>
          <Glass>
            <h3 className="text-white">GPU telemetry</h3>
            <p className="text-xl text-white mt-2">215 W · 62 °C</p>
            <p className="text-xs text-slate-500">NVIDIA power boost guard</p>
          </Glass>
        </div>
      </Frame>

      <Frame id="shot-diagnostics">
        <h2 className="text-lg font-semibold text-white mb-4">Tweak diagnostics</h2>
        <table className="w-full text-sm">
          <thead className="text-slate-500 text-xs uppercase">
            <tr>
              <th className="text-left py-2">Tweak</th>
              <th className="text-left">Detected</th>
            </tr>
          </thead>
          <tbody className="text-slate-200">
            {[
              ["Game Bar off", "on"],
              ["Processor min 100%", "off"],
              ["TCP autotuning normal", "on"],
            ].map(([n, d]) => (
              <tr key={n} className="border-t border-white/5">
                <td className="py-2">{n}</td>
                <td className="text-teal-200">{d}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </Frame>
    </div>
  );
}
