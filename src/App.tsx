import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { BenchmarkPanel, runCs2Benchmark } from "./components/BenchmarkPanel";
import { DiagnosticsPanel } from "./components/DiagnosticsPanel";
import { PerformancePanel } from "./components/PerformancePanel";
import { AutoOptimizePanel } from "./components/AutoOptimizePanel";
import { OptimizePanel } from "./components/OptimizePanel";
import { ToastStack, type ToastKind } from "./components/Toast";
import type {
  ApplyProfileResult,
  BenchmarkHistoryEntry,
  InteractiveBiosReport,
  SafeModeSession,
  Cs2CompetitiveResult,
  Cs2Optimization,
  ElevationStatus,
  InstalledGame,
  OptimizationProfile,
  SystemReport,
  TweakCatalogEntry,
} from "./types";

type Tab = "dashboard" | "system" | "optimize" | "performance" | "games" | "bios" | "benchmark" | "diagnostics";

const NAV: { id: Tab; label: string }[] = [
  { id: "dashboard", label: "Dashboard" },
  { id: "system", label: "System report" },
  { id: "optimize", label: "Optimize" },
  { id: "performance", label: "Performance" },
  { id: "games", label: "Games" },
  { id: "bios", label: "BIOS assistant" },
  { id: "benchmark", label: "Benchmark" },
  { id: "diagnostics", label: "Diagnostics" },
];

function profileLabel(p: OptimizationProfile): string {
  switch (p) {
    case "competitive":
      return "Competitive";
    case "balanced":
      return "Balanced";
    case "restoreDefaults":
      return "Restore defaults";
    default:
      return p;
  }
}

export default function App() {
  const [tab, setTab] = useState<Tab>("dashboard");
  const [report, setReport] = useState<SystemReport | null>(null);
  const [elevation, setElevation] = useState<ElevationStatus | null>(null);
  const [catalog, setCatalog] = useState<TweakCatalogEntry[]>([]);
  const [benchHistory, setBenchHistory] = useState<BenchmarkHistoryEntry[]>([]);
  const [lastBench, setLastBench] = useState<Cs2CompetitiveResult | null>(null);
  const [games, setGames] = useState<InstalledGame[]>([]);
  const [cs2, setCs2] = useState<Cs2Optimization | null>(null);
  const [bios, setBios] = useState<InteractiveBiosReport | null>(null);
  const [safeMode, setSafeMode] = useState<SafeModeSession | null>(null);
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<string>("Ready");
  const [toast, setToast] = useState<{ message: string; kind: ToastKind } | null>(null);

  const showToast = useCallback((message: string, kind: ToastKind) => {
    setToast({ message, kind });
  }, []);

  const refresh = useCallback(async () => {
    setBusy(true);
    setStatus("Scanning hardware…");
    try {
      const [elev, scanned] = await Promise.all([
        invoke<ElevationStatus>("get_elevation_status"),
        invoke<SystemReport>("scan_hardware"),
      ]);
      setElevation(elev);
      setReport(scanned);
      try {
        setCatalog(await invoke<TweakCatalogEntry[]>("get_tweak_catalog", { report: scanned }));
      } catch {
        setCatalog([]);
      }
      try {
        const hist = await invoke<{ entries: BenchmarkHistoryEntry[] }>("fetch_benchmark_history");
        setBenchHistory(hist.entries);
      } catch {
        setBenchHistory([]);
      }
      try {
        setGames(await invoke<InstalledGame[]>("list_games"));
      } catch {
        setGames([]);
      }
      try {
        setCs2(await invoke<Cs2Optimization>("get_cs2_plan", { report: scanned }));
      } catch {
        setCs2(null);
      }
      setBios(await invoke<InteractiveBiosReport>("get_interactive_bios", { report: scanned }));
      try {
        setSafeMode(await invoke<SafeModeSession | null>("safe_mode_status"));
      } catch {
        setSafeMode(null);
      }
      if (scanned.scanWarnings?.length) {
        setStatus(
          `Scan complete (${scanned.scanMethod}) · ${scanned.scanWarnings.length} warning(s)`,
        );
      } else {
        setStatus(`Scan complete (${scanned.scanMethod})`);
      }
    } catch (e) {
      setStatus(`Error: ${String(e)}`);
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const applyProfile = async (profile: OptimizationProfile) => {
    if (!report) return;
    setBusy(true);
    setStatus(`Applying ${profileLabel(profile)} profile…`);
    try {
      const result = await invoke<ApplyProfileResult>("apply_optimization_profile", {
        profile,
        report,
      });
      setStatus(
        `Applied ${result.applied.length} tweaks` +
          (result.skipped.length ? ` · ${result.skipped.length} skipped` : ""),
      );
      await refresh();
      return result;
    } catch (e) {
      setStatus(`Apply failed: ${String(e)}`);
    } finally {
      setBusy(false);
    }
  };

  const revertAll = async () => {
    setBusy(true);
    try {
      const n = await invoke<number>("revert_all_tweaks");
      setStatus(`Reverted ${n} backup session(s)`);
      await refresh();
    } catch (e) {
      setStatus(`Revert failed: ${String(e)}`);
    } finally {
      setBusy(false);
    }
  };

  const runCs2Bench = async (phase: "baseline" | "afterTweaks" | "afterBios") => {
    setBusy(true);
    try {
      const activeIds = catalog.filter((c) => c.enabled).map((c) => c.definition.id);
      const result = await runCs2Benchmark(phase, activeIds, bios?.motherboard ?? "");
      setLastBench(result);
      setBenchHistory(result.history.entries);
      setStatus(result.summary);
      const evalMsg = await invoke<string | null>("safe_mode_evaluate_benchmark", {
        avgFps: result.entry.cs2AvgFps,
        p1Fps: result.entry.cs2FpsP1,
      });
      if (evalMsg) setStatus(evalMsg);
      setSafeMode(await invoke<SafeModeSession | null>("safe_mode_status"));
    } catch (e) {
      setStatus(`CS2 benchmark failed: ${String(e)}`);
    } finally {
      setBusy(false);
    }
  };

  const scoreStyle = useMemo(
    () => ({ "--score": report?.performanceScore ?? 0 }) as React.CSSProperties,
    [report?.performanceScore],
  );

  return (
    <div className="min-h-full flex flex-col">
      <header className="border-b border-white/10 px-6 py-4 flex items-center justify-between gap-4">
        <div>
          <p className="text-xs uppercase tracking-[0.2em] text-teal-300/80">BoostX</p>
          <h1 className="text-xl font-semibold text-white">Competitive PC optimizer</h1>
          <p className="text-sm text-slate-400 mt-1">{status}</p>
        </div>
        <div className="flex items-center gap-2">
          {elevation && (
            <span
              className={`text-xs px-3 py-1 rounded-full border ${
                elevation.isElevated
                  ? "border-teal-400/40 text-teal-200 bg-teal-400/10"
                  : "border-amber-400/40 text-amber-100 bg-amber-400/10"
              }`}
            >
              {elevation.isElevated ? "Administrator" : "Standard user"}
            </span>
          )}
          {elevation && !elevation.isElevated && (
            <button
              type="button"
              className="px-3 py-2 rounded-lg bg-amber-500/90 hover:bg-amber-400 text-slate-900 text-sm font-semibold"
              onClick={() => void invoke("request_elevation")}
            >
              Run as administrator
            </button>
          )}
          <button
            type="button"
            disabled={busy}
            onClick={() => void refresh()}
            className="px-4 py-2 rounded-lg bg-indigo-500/90 hover:bg-indigo-400 text-white text-sm font-medium disabled:opacity-50"
          >
            Rescan
          </button>
        </div>
      </header>

      <div className="flex flex-1 min-h-0">
        <nav className="w-56 shrink-0 border-r border-white/10 p-4 space-y-1 bg-[#0a0d14]/50">
          {NAV.map((item) => (
            <button
              key={item.id}
              type="button"
              onClick={() => setTab(item.id)}
              className={`w-full text-left px-3 py-2 rounded-lg text-sm transition ${
                tab === item.id
                  ? "bg-white/10 text-white"
                  : "text-slate-400 hover:text-slate-200 hover:bg-white/5"
              }`}
            >
              {item.label}
            </button>
          ))}
        </nav>

        <main className="flex-1 overflow-auto p-6 space-y-6">
          {report?.isDemoData && (
            <div className="glass rounded-xl p-4 text-sm text-amber-100 border-amber-400/30">
              Demo data — run BoostX on Windows with admin rights for live WMI/registry tweaks.
            </div>
          )}
          {report?.scanWarnings && report.scanWarnings.length > 0 && (
            <div className="glass rounded-xl p-4 text-sm text-amber-100 border-amber-400/30 space-y-2">
              <p className="font-medium">Scan notes ({report.scanMethod})</p>
              <ul className="list-disc pl-5 text-amber-100/90 space-y-1">
                {report.scanWarnings.map((w) => (
                  <li key={w}>{w}</li>
                ))}
              </ul>
            </div>
          )}
          {safeMode && !safeMode.confirmed && (
            <div className="glass rounded-xl p-4 text-sm border border-teal-400/30 flex flex-col md:flex-row md:items-center gap-3 justify-between">
              <div>
                <p className="font-medium text-teal-100">Safe mode active</p>
                <p className="text-slate-400 mt-1">
                  Run a CS2 benchmark after tweaks — regression vs baseline auto-reverts.{" "}
                  {safeMode.baselineAvgFps != null
                    ? `Baseline: ${safeMode.baselineAvgFps.toFixed(0)} avg / ${safeMode.baselineP1Fps?.toFixed(0) ?? "—"} 1% lows.`
                    : "Run a baseline benchmark first for FPS-based auto-revert."}
                </p>
              </div>
              <button
                type="button"
                disabled={busy}
                onClick={async () => {
                  await invoke("safe_mode_confirm");
                  setSafeMode(null);
                  setStatus("Safe mode cleared — tweaks kept.");
                }}
                className="px-4 py-2 rounded-lg bg-teal-500 text-slate-900 font-semibold text-sm disabled:opacity-50 shrink-0"
              >
                Keep tweaks
              </button>
            </div>
          )}

          {tab === "dashboard" && report && (
            <section className="space-y-6">
              <AutoOptimizePanel report={report} busy={busy} onComplete={() => void refresh()} />
              <div className="grid md:grid-cols-3 gap-3">
                {[
                  ["1", "Scan your PC", "Hardware score and CS2-ready report."],
                  ["2", "Optimize", "One-click Competitive preset or pick toggles."],
                  ["3", "Benchmark CS2", "Measure FPS gains vs baseline."],
                ].map(([n, title, desc]) => (
                  <div key={n} className="rounded-2xl border border-white/10 bg-[#0c1018]/70 p-4">
                    <span className="text-teal-400 text-sm font-semibold">Step {n}</span>
                    <h3 className="text-white font-medium mt-1">{title}</h3>
                    <p className="text-sm text-slate-500 mt-1">{desc}</p>
                  </div>
                ))}
              </div>
            <div className="grid md:grid-cols-[240px_1fr] gap-6">
              <div
                className="score-ring rounded-full w-56 h-56 mx-auto flex items-center justify-center shadow-[0_0_40px_rgba(94,234,212,0.15)]"
                style={scoreStyle}
              >
                <div className="w-44 h-44 rounded-full bg-[var(--color-bg-1)] flex flex-col items-center justify-center border border-white/10">
                  <span className="text-4xl font-bold text-white">{report.performanceScore}</span>
                  <span className="text-xs uppercase tracking-widest text-slate-400">Score</span>
                </div>
              </div>
              <div className="space-y-4">
                <div className="glass rounded-xl p-5">
                  <h2 className="text-lg font-semibold text-white mb-2">Quick actions</h2>
                  <p className="text-sm text-slate-400 mb-4">
                    Creates a System Restore point and backs up registry values before each profile run.
                  </p>
                  <div className="flex flex-wrap gap-2">
                    <button
                      type="button"
                      disabled={busy}
                      onClick={() => void applyProfile("competitive")}
                      className="px-4 py-2 rounded-lg bg-teal-500 text-slate-900 font-semibold text-sm disabled:opacity-50"
                    >
                      Competitive
                    </button>
                    <button
                      type="button"
                      disabled={busy}
                      onClick={() => void applyProfile("balanced")}
                      className="px-4 py-2 rounded-lg bg-indigo-500 text-white text-sm disabled:opacity-50"
                    >
                      Balanced
                    </button>
                    <button
                      type="button"
                      disabled={busy}
                      onClick={() => void revertAll()}
                      className="px-4 py-2 rounded-lg border border-rose-400/40 text-rose-200 text-sm disabled:opacity-50"
                    >
                      Revert all
                    </button>
                  </div>
                </div>
                <div className="grid sm:grid-cols-2 gap-3">
                  {report.scoreBreakdown.map((f) => (
                    <div key={f.label} className="glass rounded-lg p-4">
                      <div className="flex justify-between text-sm">
                        <span className="text-slate-300">{f.label}</span>
                        <span className="text-teal-300">{f.points}/{f.maxPoints}</span>
                      </div>
                      <p className="text-xs text-slate-500 mt-2 line-clamp-2">{f.note}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>
            </section>
          )}

          {tab === "system" && report && (
            <section className="grid lg:grid-cols-2 gap-4">
              <ReportCard title="CPU" lines={[report.cpu.model, `${report.cpu.vendor} · ${report.cpu.generationHint}`, `${report.cpu.cores}c / ${report.cpu.logicalProcessors}t`]} />
              <ReportCard title="GPU" lines={[report.gpu.model, report.gpu.driverVersion ? `Driver ${report.gpu.driverVersion}` : "Driver unknown"]} />
              <ReportCard
                title="Memory"
                lines={[
                  `${report.ram.totalGb.toFixed(0)} GB`,
                  `Running ${report.ram.speedMhz ?? "?"} MHz · Rated ${report.ram.ratedSpeedMhz ?? "?"}`,
                  report.ram.runningBelowRated ? "Below rated speed — enable XMP/EXPO in BIOS" : "Speed OK",
                ]}
              />
              <ReportCard title="Windows" lines={[report.windows.version, `Build ${report.windows.build}`]} />
              <ReportCard
                title="Motherboard"
                lines={[
                  `${report.motherboard.manufacturer} ${report.motherboard.model}`,
                  report.motherboard.biosVersion ? `BIOS ${report.motherboard.biosVersion}` : "",
                ]}
              />
              <ReportCard
                title="Display"
                lines={report.monitors.map(
                  (m) => `${m.name} · ${m.resolution ?? "?"} @ ${m.refreshHz ?? "?"} Hz`,
                )}
              />
              <ReportCard
                title="Storage"
                lines={report.storage.map((d) => `${d.kind}: ${d.model} (${d.sizeGb.toFixed(0)} GB)`)}
              />
            </section>
          )}

          {tab === "performance" && <PerformancePanel busy={busy} />}

          {tab === "optimize" && (
            <OptimizePanel
              report={report}
              catalog={catalog}
              busy={busy}
              onRefresh={refresh}
              onToast={showToast}
              onApplyProfile={async (p) => {
                await applyProfile(p);
              }}
            />
          )}

          {tab === "games" && (
            <section className="space-y-6">
              <div>
                <h2 className="text-lg font-semibold text-white mb-3">Detected games</h2>
                <div className="grid sm:grid-cols-2 gap-3">
                  {games.map((g) => (
                    <div key={g.name} className="glass rounded-lg p-4">
                      <p className="font-medium text-white">{g.name}</p>
                      <p className="text-xs text-slate-500 mt-1">{g.platform} · App {g.appId}</p>
                    </div>
                  ))}
                </div>
              </div>
              {cs2 && (
                <div className="glass rounded-xl p-5 space-y-4">
                  <h2 className="text-lg font-semibold text-white">Counter-Strike 2</h2>
                  <div>
                    <p className="text-xs uppercase tracking-wider text-slate-500 mb-1">Launch options</p>
                    <code className="block text-sm bg-black/30 rounded p-3 text-teal-200 break-all">
                      {cs2.launchOptions}
                    </code>
                  </div>
                  <div>
                    <p className="text-xs uppercase tracking-wider text-slate-500 mb-1">autoexec (boostx_autoexec.cfg)</p>
                    <pre className="text-xs bg-black/30 rounded p-3 overflow-x-auto text-slate-300">{cs2.autoexec}</pre>
                  </div>
                  <ul className="text-sm text-slate-400 list-disc pl-5 space-y-1">
                    {cs2.notes.map((n) => (
                      <li key={n}>{n}</li>
                    ))}
                  </ul>
                </div>
              )}
            </section>
          )}

          {tab === "bios" && bios && (
            <section className="space-y-4">
              <div className="glass rounded-xl p-5">
                <h2 className="text-lg font-semibold text-white">{bios.vendor} BIOS assistant</h2>
                <p className="text-sm text-slate-400 mt-1">
                  {bios.motherboard} — BoostX never writes firmware; follow these steps manually.
                </p>
                {bios.postRebootVerificationPending && (
                  <p className="text-sm text-amber-200 mt-3 border border-amber-500/30 rounded-lg px-3 py-2 bg-amber-500/10">
                    Post-reboot verification pending — rescan after changing XMP/EXPO in firmware.
                  </p>
                )}
                <button
                  type="button"
                  disabled={busy || !report}
                  onClick={async () => {
                    if (!report) return;
                    try {
                      await invoke("reboot_to_bios", { report });
                    } catch (e) {
                      setStatus(String(e));
                    }
                  }}
                  className="mt-4 px-4 py-2 rounded-lg bg-rose-500/90 hover:bg-rose-400 text-white text-sm disabled:opacity-50"
                >
                  Reboot to firmware ({bios.rebootToFirmwareCommand})
                </button>
              </div>
              {bios.settings.map((item) => (
                <div key={item.id} className="glass rounded-xl p-4">
                  <div className="flex flex-wrap items-center gap-2">
                    <h3 className="font-medium text-white">{item.title}</h3>
                    <span className="text-[10px] uppercase px-2 py-0.5 rounded bg-white/5">{item.risk} risk</span>
                    <span className="text-[10px] uppercase px-2 py-0.5 rounded bg-white/5">{item.detectedStatus}</span>
                    {item.verifiedAfterReboot && (
                      <span className="text-[10px] uppercase px-2 py-0.5 rounded bg-teal-500/20 text-teal-100">
                        Verified after reboot
                      </span>
                    )}
                  </div>
                  <p className="text-sm text-teal-200 mt-2">{item.competitiveFpsNote}</p>
                  <p className="text-sm text-slate-300 mt-1">{item.explanation}</p>
                  <p className="text-sm text-slate-400 mt-1">Set: {item.recommendedValue}</p>
                  <p className="text-sm text-slate-400">Menu: {item.menuPath}</p>
                  <p className="text-sm text-slate-500 mt-1">{item.detectedDetail}</p>
                </div>
              ))}
            </section>
          )}

          {tab === "diagnostics" && <DiagnosticsPanel report={report} />}

          {tab === "benchmark" && (
            <BenchmarkPanel
              report={report}
              history={benchHistory}
              busy={busy}
              onRun={runCs2Bench}
              lastResult={lastBench}
            />
          )}
        </main>
      </div>
      <ToastStack
        message={toast?.message ?? null}
        kind={toast?.kind ?? "info"}
        onClear={() => setToast(null)}
      />
    </div>
  );
}

function ReportCard({ title, lines }: { title: string; lines: string[] }) {
  return (
    <div className="glass rounded-xl p-4">
      <h3 className="text-sm uppercase tracking-wider text-slate-500">{title}</h3>
      <ul className="mt-2 space-y-1 text-sm text-slate-200">
        {lines.filter(Boolean).map((line) => (
          <li key={line}>{line}</li>
        ))}
      </ul>
    </div>
  );
}
