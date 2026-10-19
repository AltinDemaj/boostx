import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

type Props = { busy: boolean };

export function PerformancePanel({ busy }: Props) {
  const [timerUs, setTimerUs] = useState<number | null>(null);
  const [gpu, setGpu] = useState<string>("—");

  useEffect(() => {
    void invoke<number>("get_timer_resolution_us").then(setTimerUs).catch(() => setTimerUs(null));
    void invoke<{ tempC?: number; powerWatts?: number }>("get_gpu_telemetry_sample")
      .then((s) => setGpu(`${s.powerWatts ?? "?"} W · ${s.tempC ?? "?"} °C`))
      .catch(() => setGpu("NVIDIA telemetry unavailable"));
  }, []);

  return (
    <section className="grid md:grid-cols-2 gap-4">
      <div className="glass rounded-xl p-5">
        <h2 className="text-lg font-semibold text-white">Timer resolution</h2>
        <p className="text-3xl font-mono text-teal-300 mt-3">
          {timerUs != null ? `${timerUs.toFixed(1)} µs` : "—"}
        </p>
        <button
          type="button"
          disabled={busy}
          onClick={() =>
            void invoke("start_timer_service_cmd", {
              policy: { targetResolutionMs: 0.5, useGlobalRegistry: true },
            })
          }
          className="mt-4 px-4 py-2 rounded-lg bg-teal-500 text-slate-900 text-sm font-semibold disabled:opacity-50"
        >
          Start 0.5 ms timer service
        </button>
      </div>
      <div className="glass rounded-xl p-5">
        <h2 className="text-lg font-semibold text-white">GPU telemetry</h2>
        <p className="text-xl text-white mt-3">{gpu}</p>
        <p className="text-xs text-slate-500 mt-2">Requires NVIDIA driver + nvidia-smi on Windows.</p>
      </div>
    </section>
  );
}
