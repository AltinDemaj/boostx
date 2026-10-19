import type {
  BenchmarkHistoryEntry,
  Cs2CompetitiveResult,
  SystemReport,
  TweakCatalogEntry,
} from "../types";

export const demoReport: SystemReport = {
  cpu: {
    vendor: "AMD",
    model: "AMD Ryzen 7 7800X3D",
    cores: 8,
    logicalProcessors: 16,
    generationHint: "Zen 4",
    baseClockMhz: 4200,
  },
  gpu: {
    vendor: "NVIDIA",
    model: "NVIDIA GeForce RTX 4070 SUPER",
    vramMb: 12288,
    driverVersion: "581.29",
  },
  ram: {
    totalGb: 32,
    speedMhz: 6000,
    ratedSpeedMhz: 6000,
    xmpProfileDetected: true,
    runningBelowRated: false,
  },
  storage: [{ model: "Samsung SSD 990 PRO 1TB", kind: "NVMe", sizeGb: 1000, freeGb: 620 }],
  motherboard: {
    manufacturer: "ASUS",
    model: "ROG STRIX B650-E",
    biosVersion: "3222",
    biosVendor: "American Megatrends",
  },
  monitors: [{ name: "LG 27GP850", refreshHz: 180, resolution: "2560x1440" }],
  windows: { version: "Windows 11 Pro", build: "26100", edition: "Professional" },
  performanceScore: 91,
  scoreBreakdown: [
    { label: "CPU", points: 24, maxPoints: 25, note: "Strong single-thread for CS2" },
    { label: "GPU", points: 26, maxPoints: 30, note: "RTX 40-series latency friendly" },
    { label: "Memory", points: 14, maxPoints: 15, note: "Running rated DDR5 speed" },
    { label: "Display", points: 10, maxPoints: 10, note: "180 Hz competitive refresh" },
    { label: "Storage", points: 9, maxPoints: 10, note: "NVMe OS drive" },
    { label: "Firmware", points: 8, maxPoints: 10, note: "EXPO enabled" },
  ],
  scannedAt: new Date().toISOString(),
  isDemoData: true,
  scanMethod: "landing-capture",
  scanWarnings: [],
};

function tweak(
  partial: Omit<TweakCatalogEntry["definition"], "profiles" | "requiresReboot" | "guidanceOnly"> & {
    detected: TweakCatalogEntry["detected"];
    enabled: boolean;
  },
): TweakCatalogEntry {
  return {
    definition: {
      ...partial,
      profiles: ["competitive"],
      requiresReboot: false,
      guidanceOnly: false,
    },
    state: { id: partial.id, applied: partial.enabled, appliedAt: partial.enabled ? new Date().toISOString() : undefined },
    detected: partial.detected,
    enabled: partial.enabled,
  };
}

export const demoCatalog: TweakCatalogEntry[] = [
  tweak({
    id: "power_ultimate",
    name: "Ultimate Performance power plan",
    description: "Activates Ultimate Performance for lowest latency on AC.",
    expectedBenefit: "Steadier frame times under CPU load.",
    category: "powerCpu",
    risk: "low",
    impact: "medium",
    detected: "on",
    enabled: true,
  }),
  tweak({
    id: "power_min_processor_100",
    name: "Minimum processor state 100% (AC)",
    description: "Sets minimum CPU state to 100% on AC.",
    expectedBenefit: "Lower 1% lows in CPU-bound titles.",
    category: "powerCpu",
    risk: "medium",
    impact: "medium",
    detected: "on",
    enabled: true,
  }),
  tweak({
    id: "net_tcp_autotun_normal",
    name: "TCP autotuning level — normal",
    description: "netsh autotuninglevel=normal",
    expectedBenefit: "Predictable TCP window growth.",
    category: "network",
    risk: "low",
    impact: "medium",
    detected: "on",
    enabled: true,
  }),
  tweak({
    id: "defender_cs2_exclusion",
    name: "Windows Defender — CS2 folder exclusion",
    description: "Adds CS2 install folder to Defender exclusions.",
    expectedBenefit: "Avoids scan spikes during match loads.",
    category: "gameSpecific",
    risk: "medium",
    impact: "low",
    detected: "on",
    enabled: true,
  }),
  tweak({
    id: "hags",
    name: "Hardware-accelerated GPU scheduling (HAGS)",
    description: "Enables HAGS for lower scheduling latency.",
    expectedBenefit: "Reduced GPU queue delay.",
    category: "gpu",
    risk: "low",
    impact: "medium",
    detected: "on",
    enabled: true,
  }),
  tweak({
    id: "mouse_accel_off",
    name: "Disable mouse acceleration",
    description: "1:1 pointer for competitive aim.",
    expectedBenefit: "Consistent flicks.",
    category: "input",
    risk: "low",
    impact: "high",
    detected: "off",
    enabled: false,
  }),
];

export const demoBenchHistory: BenchmarkHistoryEntry[] = [
  {
    id: "b1",
    capturedAt: new Date(Date.now() - 86400000).toISOString(),
    phase: "baseline",
    label: "baseline",
    source: "cs2_live",
    frameSampleCount: 2840,
    captureDurationSecs: 45.2,
    runId: "demo-baseline",
    cs2AvgFps: 268.4,
    cs2FpsP1: 221.2,
    cs2FpsP01: 198.6,
    frametimeJitterMs: 1.12,
    systemLatencyScore: 88,
    timerLatencyUs: 500,
    inputPathScore: 92,
    cpuSingleThreadScore: 94,
    memoryLatencyUs: 68.2,
    activeTweakIds: [],
    biosSummary: "",
    details: { cpuSingle: 94, cpuMulti: 88, memoryBandwidth: 72, storageReadMbps: 81, storageWriteMbps: 76 },
  },
  {
    id: "b2",
    capturedAt: new Date().toISOString(),
    phase: "afterTweaks",
    label: "afterTweaks",
    source: "cs2_live",
    frameSampleCount: 2912,
    captureDurationSecs: 45.0,
    runId: "demo-after",
    cs2AvgFps: 287.9,
    cs2FpsP1: 241.5,
    cs2FpsP01: 212.3,
    frametimeJitterMs: 0.94,
    systemLatencyScore: 91,
    timerLatencyUs: 500,
    inputPathScore: 94,
    cpuSingleThreadScore: 94,
    memoryLatencyUs: 67.8,
    activeTweakIds: ["power_ultimate", "power_min_processor_100"],
    biosSummary: "",
    details: { cpuSingle: 94, cpuMulti: 88, memoryBandwidth: 72, storageReadMbps: 81, storageWriteMbps: 76 },
  },
];

export const demoLastBench: Cs2CompetitiveResult = {
  entry: demoBenchHistory[1],
  history: { entries: demoBenchHistory },
  deltaAvgFpsVsPrevious: 19.5,
  deltaP1FpsVsPrevious: 20.3,
  deltaAvgFpsVsStarting: 19.5,
  deltaP1FpsVsStarting: 20.3,
  summary: "CS2 live (PresentMon) · 2912 frametime samples · 45.0s capture · vs previous run: avg +19.5, 1% +20.3",
  pipelineLog: {
    runId: "demo-after",
    startedAt: new Date().toISOString(),
    finishedAt: new Date().toISOString(),
    source: "cs2_live",
    steps: [
      "[12:00:01.000] Benchmark run started — fresh capture",
      "[12:00:02.100] CS2 install: C:\\Program Files (x86)\\Steam\\steamapps\\common\\Counter-Strike Global Offensive\\game\\bin\\win64\\cs2.exe",
      "[12:00:45.200] Parsed 2912 frametime samples from CSV",
    ],
  },
};

export type DemoLiveTick = {
  elapsedMs: number;
  instantFps: number;
  frametimeMs: number;
  isSpike: boolean;
  cpuUsagePct?: number;
  gpuUsagePct?: number;
  gpuTempC?: number;
};

export function buildDemoLiveTicks(): DemoLiveTick[] {
  const ticks: DemoLiveTick[] = [];
  let t = 0;
  for (let i = 0; i < 80; i++) {
    const spike = i % 17 === 0;
    const ft = spike ? 28 + (i % 3) : 3.2 + (Math.sin(i / 4) * 0.4) + (i % 5) * 0.05;
    ticks.push({
      elapsedMs: t,
      instantFps: 1000 / ft,
      frametimeMs: ft,
      isSpike: spike,
      cpuUsagePct: 42 + (i % 20),
      gpuUsagePct: 78 + (i % 12),
      gpuTempC: 62 + (i % 8) * 0.25,
    });
    t += Math.round(ft);
  }
  return ticks;
}

export const demoDiagnostics = {
  elevated: true,
  manifestHint: "requireAdministrator embedded in PE",
  generatedAt: new Date().toISOString(),
  rows: [
    { id: "power_min_processor_100", name: "Minimum processor state 100% (AC)", detected: "on", note: "" },
    { id: "power_max_processor_100", name: "Maximum processor state 100% (AC)", detected: "on", note: "" },
    { id: "amd_processor_boost", name: "AMD processor performance boost mode", detected: "on", note: "" },
    { id: "defender_cs2_exclusion", name: "Windows Defender — CS2 folder exclusion", detected: "on", note: "" },
    { id: "net_tcp_autotun_normal", name: "TCP autotuning level — normal", detected: "on", note: "" },
    { id: "nvidia_guidance", name: "NVIDIA Control Panel — competitive profile", detected: "n/a", note: "Guidance only" },
  ],
};

export const demoPerformancePreview = {
  topo: {
    performanceCores: [0, 1, 2, 3, 4, 5, 6, 7],
    efficientCores: [] as number[],
    note: "Ryzen 7800X3D — single CCD, all cores performance class",
  },
  watcher: {
    running: true,
    config: {
      enabled: true,
      processNames: ["cs2.exe"],
      usePerformanceCoresOnly: true,
      priorityClass: "high",
    },
    lastAction: "Affinity applied to cs2.exe (pid 18424)",
  },
  timerUs: 500,
  gpu: {
    vendor: "NVIDIA",
    minWatts: 150,
    maxWatts: 250,
    currentWatts: 198,
    temperatureC: 64,
  },
};
