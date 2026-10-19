import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { SystemReport } from "../types";

type DiagnosticsReport = {
  generatedAt: string;
  summary: string;
  rows: Array<{
    id: string;
    name: string;
    detected: string;
    applicable: boolean;
    note: string;
  }>;
};

type Props = { report: SystemReport | null };

export function DiagnosticsPanel({ report }: Props) {
  const [diag, setDiag] = useState<DiagnosticsReport | null>(null);

  useEffect(() => {
    if (!report) return;
    void invoke<DiagnosticsReport>("run_tweak_diagnostics", { report }).then(setDiag);
  }, [report]);

  if (!diag) {
    return <p className="text-slate-400 text-sm">Run a hardware scan to load diagnostics.</p>;
  }

  return (
    <section className="glass rounded-xl p-5 space-y-3">
      <h2 className="text-lg font-semibold text-white">Tweak diagnostics</h2>
      <p className="text-sm text-slate-400">{diag.summary}</p>
      <div className="overflow-auto max-h-[60vh]">
        <table className="w-full text-sm text-left">
          <thead className="text-slate-500 uppercase text-xs">
            <tr>
              <th className="py-2">Tweak</th>
              <th>Detected</th>
              <th>Note</th>
            </tr>
          </thead>
          <tbody>
            {diag.rows.map((r) => (
              <tr key={r.id} className="border-t border-white/5">
                <td className="py-2 text-slate-200">{r.name}</td>
                <td className="text-teal-200">{r.detected}</td>
                <td className="text-slate-500">{r.note}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
