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
      {/* truncated for tool size - will fail if truncated */}
    </div>
  );
}
