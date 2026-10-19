import { invoke } from "@tauri-apps/api/core";
import type { BenchmarkHistoryEntry, Cs2CompetitiveResult, SystemReport } from "../types";

type Props = {
  report: SystemReport | null;
  history: BenchmarkHistoryEntry[];
  busy: boolean;
  onRun: (phase: "baseline" | "afterTweaks" | "afterBios") => Promise<void>;
  lastResult: Cs2CompetitiveResult | null;
};

export async function runCs2Benchmark(
  phase: "baseline" | "afterTweaks" | "afterBios",
  activeTweakIds: string[],
  biosSummary: string,
): Promise<Cs2CompetitiveResult> {
  return invoke<Cs2CompetitiveResult>("run_cs2_competitive_benchmark", {
    phase,
    label: phase,
    activeTweakIds,
    biosSummary,
  });
}

export function BenchmarkPanel({ history, busy, onRun, lastResult }: Props) {
  return (
    <section className="space-y-4">
      <div className="glass rounded-xl p-5">
        <h2 className="text-lg font-semibold text-white">CS2 competitive benchmark</h2>
        <p className="text-sm text-slate-400 mt-1">
          PresentMon live capture when CS2 is running; micro-bench fallback on CI/headless hosts.
        </p>
        <div className="flex flex-wrap gap-2 mt-4">
          {(
            [
              ["baseline", "Baseline"],
              ["afterTweaks", "After tweaks"],
              ["afterBios", "After BIOS"],
            ] as const
          ).map(([phase, label]) => (
            <button
              key={phase}
              type="button"
              disabled={busy}
              onClick={() => void onRun(phase)}
              className="px-4 py-2 rounded-lg bg-indigo-500/90 text-white text-sm disabled:opacity-50"
            >
              {label}
            </button>
          ))}
        </div>
        {lastResult && (
          <div className="mt-4 rounded-lg bg-black/30 p-4 text-sm">
            <p className="text-teal-200">{lastResult.summary}</p>
            <p className="text-white font-mono mt-2">
              {lastResult.entry.cs2AvgFps.toFixed(1)} avg · {lastResult.entry.cs2FpsP1.toFixed(1)} 1% · jitter{" "}
              {lastResult.entry.frametimeJitterMs.toFixed(2)} ms
            </p>
          </div>
        )}
      </div>
      <div className="glass rounded-xl p-5">
        <h3 className="text-sm uppercase tracking-wider text-slate-500 mb-3">History</h3>
        <ul className="text-sm space-y-2 max-h-64 overflow-auto">
          {history.map((h) => (
            <li key={h.id} className="flex justify-between gap-4 border-b border-white/5 pb-2">
              <span className="text-slate-300">{h.label}</span>
              <span className="font-mono text-teal-200">
                {h.cs2AvgFps.toFixed(0)} / {h.cs2FpsP1.toFixed(0)} 1%
              </span>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
