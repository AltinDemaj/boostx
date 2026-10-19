import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "react";
import type { AutoOptimizeProgress, AutoOptimizeReport, SystemReport } from "../types";

type Props = {
  report: SystemReport | null;
  busy: boolean;
  onComplete: () => void;
};

export function AutoOptimizePanel({ report, busy, onComplete }: Props) {
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<AutoOptimizeProgress | null>(null);
  const [lastReport, setLastReport] = useState<AutoOptimizeReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [graph, setGraph] = useState<number[]>([]);

  useEffect(() => {
    void invoke<AutoOptimizeReport | null>("fetch_auto_optimize_report").then((r) => {
      if (r) setLastReport(r);
    });
  }, []);

  useEffect(() => {
    let unlistenP: (() => void) | undefined;
    let unlistenC: (() => void) | undefined;
    let unlistenE: (() => void) | undefined;
    void listen<AutoOptimizeProgress>("auto-optimize-progress", (ev) => {
      setProgress(ev.payload);
      setGraph((g) => [...g.slice(-80), ev.payload.liveFps]);
    }).then((fn) => {
      unlistenP = fn;
    });
    void listen<AutoOptimizeReport>("auto-optimize-complete", (ev) => {
      setLastReport(ev.payload);
      setRunning(false);
      setProgress(null);
      onComplete();
    }).then((fn) => {
      unlistenC = fn;
    });
    void listen<string>("auto-optimize-error", (ev) => {
      setError(ev.payload);
      setRunning(false);
    }).then((fn) => {
      unlistenE = fn;
    });
    return () => {
      unlistenP?.();
      unlistenC?.();
      unlistenE?.();
    };
  }, [onComplete]);

  const maxFps = useMemo(() => Math.max(...graph, 1), [graph]);

  const start = async () => {
    if (!report) return;
    setError(null);
    setRunning(true);
    setGraph([]);
    try {
      await invoke("start_auto_optimize", { report });
    } catch (e) {
      setError(String(e));
      setRunning(false);
    }
  };

  const cancel = async () => {
    await invoke("cancel_auto_optimize_cmd");
    setRunning(false);
    setProgress(null);
    onComplete();
  };

  return (
    <section className="hero-auto space-y-4">
      <div className="flex flex-col lg:flex-row lg:items-center gap-4 justify-between">
        <div>
          <h2 className="text-2xl font-bold text-white tracking-tight">Auto Optimize</h2>
          <p className="text-sm text-slate-400 mt-1 max-w-xl">
            Baseline benchmark → test each competitive tweak with short multi-run samples → keep winners, revert
            regressions → final report card on your PC.
          </p>
        </div>
        <div className="flex gap-2 shrink-0">
          <button
            type="button"
            disabled={busy || running || !report}
            onClick={() => void start()}
            className="btn-hero px-8 py-3 rounded-xl text-base font-bold disabled:opacity-40"
          >
            {running ? "Running…" : "Auto Optimize"}
          </button>
          {running && (
            <button
              type="button"
              onClick={() => void cancel()}
              className="px-4 py-3 rounded-xl border border-rose-400/50 text-rose-200 text-sm"
            >
              Cancel
            </button>
          )}
        </div>
      </div>

      {error && <p className="text-sm text-rose-300">{error}</p>}

      {progress && (
        <div className="glass rounded-2xl p-4 space-y-3 border border-cyan-400/20">
          <div className="flex justify-between text-sm">
            <span className="text-cyan-200 capitalize">{progress.phase}</span>
            <span className="text-slate-500">
              {progress.current}/{progress.total || "—"} · ETA ~{progress.etaSecs}s
            </span>
          </div>
          <p className="text-white text-sm">{progress.message}</p>
          <div className="h-24 flex items-end gap-0.5 bg-black/30 rounded-lg p-2">
            {graph.map((v, i) => (
              <div
                key={i}
                className="flex-1 bg-gradient-to-t from-cyan-500 to-violet-400 rounded-sm min-w-[2px]"
                style={{ height: `${(v / maxFps) * 100}%` }}
              />
            ))}
          </div>
          <p className="text-xs text-slate-500">
            Live: {progress.liveFps.toFixed(1)} FPS · jitter {progress.liveJitterMs.toFixed(2)} ms
          </p>
        </div>
      )}

      {lastReport && !running && (
        <div className="glass rounded-2xl p-5 border border-white/10 space-y-3">
          <h3 className="text-lg font-semibold text-white">Last report card</h3>
          <p className="text-sm text-slate-400">{lastReport.summary}</p>
          <div className="grid sm:grid-cols-2 gap-3 text-sm">
            <div className="rounded-xl bg-black/30 p-3">
              <p className="text-slate-500">Baseline</p>
              <p className="text-white font-mono">
                {lastReport.baseline.avgFps.toFixed(1)} avg · {lastReport.baseline.p1Fps.toFixed(1)} 1%
              </p>
            </div>
            <div className="rounded-xl bg-black/30 p-3">
              <p className="text-slate-500">Final</p>
              <p className="text-teal-300 font-mono">
                {lastReport.finalStats.avgFps.toFixed(1)} avg · {lastReport.finalStats.p1Fps.toFixed(1)} 1%
              </p>
            </div>
          </div>
          <ul className="text-xs text-slate-400 space-y-1 max-h-40 overflow-auto">
            {lastReport.trials.slice(0, 12).map((t) => (
              <li key={t.tweakId}>
                <span className="text-slate-300">{t.tweakName}</span> — {t.verdict} ({t.deltaAvgFps >= 0 ? "+" : ""}
                {t.deltaAvgFps.toFixed(1)} avg)
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}
