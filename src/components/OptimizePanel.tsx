import { useMemo, useState } from "react";
import type { OptimizationProfile, SystemReport, TweakCatalogEntry } from "../types";
import { CATEGORY_LABELS } from "../types";
import { toggleTweak } from "../lib/toggleTweak";

type Props = {
  report: SystemReport | null;
  catalog: TweakCatalogEntry[];
  busy: boolean;
  onRefresh: () => void;
  onToast: (message: string, kind: "info" | "success" | "error") => void;
  onApplyProfile: (profile: OptimizationProfile) => Promise<void>;
};

function impactClass(level: string): string {
  if (level === "high") return "impact-pill-high";
  if (level === "medium") return "impact-pill-med";
  return "impact-pill-low";
}

export function OptimizePanel({ report, catalog, busy, onRefresh, onToast, onApplyProfile }: Props) {
  const [filter, setFilter] = useState("");

  const grouped = useMemo(() => {
    const q = filter.toLowerCase();
    const items = catalog.filter(
      (c) =>
        !q ||
        c.definition.name.toLowerCase().includes(q) ||
        c.definition.id.toLowerCase().includes(q),
    );
    const map = new Map<string, TweakCatalogEntry[]>();
    for (const entry of items) {
      const cat = entry.definition.category;
      if (!map.has(cat)) map.set(cat, []);
      map.get(cat)!.push(entry);
    }
    return map;
  }, [catalog, filter]);

  const toggle = async (id: string, enable: boolean) => {
    if (!report) return;
    try {
      const res = await toggleTweak(id, enable, report);
      onToast(res.message, res.verified ? "success" : "info");
      onRefresh();
    } catch (e) {
      onToast(String(e), "error");
    }
  };

  return (
    <section className="space-y-4">
      <div className="flex flex-wrap gap-2 items-center justify-between">
        <h2 className="text-lg font-semibold text-white">Tweak catalog</h2>
        <input
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          placeholder="Search tweaks…"
          className="px-3 py-2 rounded-lg bg-black/30 border border-white/10 text-sm text-white w-64"
        />
      </div>
      <div className="flex flex-wrap gap-2">
        {(["competitive", "balanced", "restoreDefaults"] as OptimizationProfile[]).map((p) => (
          <button
            key={p}
            type="button"
            disabled={busy || !report}
            onClick={() => void onApplyProfile(p)}
            className="px-3 py-2 rounded-lg bg-white/10 hover:bg-white/15 text-sm text-white disabled:opacity-50"
          >
            Apply {p}
          </button>
        ))}
      </div>
      {Array.from(grouped.entries()).map(([cat, entries]) => (
        <div key={cat} className="glass rounded-xl p-4 space-y-3">
          <h3 className="text-sm uppercase tracking-wider text-slate-500">
            {CATEGORY_LABELS[cat as keyof typeof CATEGORY_LABELS] ?? cat}
          </h3>
          <div className="space-y-2">
            {entries.map((entry) => (
              <div
                key={entry.definition.id}
                className="flex flex-col md:flex-row md:items-center gap-3 border border-white/5 rounded-lg p-3 bg-black/20"
              >
                <div className="flex-1 min-w-0">
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="font-medium text-white">{entry.definition.name}</p>
                    <span className={`impact-meter ${impactClass(entry.definition.impact)}`}>
                      FPS {entry.definition.impact} · Latency {entry.definition.impact}
                    </span>
                    <span className="text-[10px] uppercase px-2 py-0.5 rounded bg-white/5 text-slate-400">
                      {entry.detected}
                    </span>
                  </div>
                  <p className="text-xs text-slate-400 mt-1">{entry.definition.description}</p>
                  <p className="text-xs text-teal-200/90 mt-1">{entry.definition.expectedBenefit}</p>
                </div>
                <button
                  type="button"
                  disabled={busy || !report || entry.definition.guidanceOnly}
                  onClick={() => void toggle(entry.definition.id, !entry.enabled)}
                  className={`px-4 py-2 rounded-lg text-sm font-semibold shrink-0 disabled:opacity-40 ${
                    entry.enabled
                      ? "bg-rose-500/80 text-white"
                      : "bg-teal-500 text-slate-900"
                  }`}
                >
                  {entry.definition.guidanceOnly ? "Guide" : entry.enabled ? "Revert" : "Apply"}
                </button>
              </div>
            ))}
          </div>
        </div>
      ))}
    </section>
  );
}
