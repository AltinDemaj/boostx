export type OptimizationProfile = "competitive" | "balanced" | "restoreDefaults";

export type TweakCategory =
  | "powerCpu"
  | "gpu"
  | "latencyTimers"
  | "network"
  | "input"
  | "servicesBloat"
  | "memory"
  | "storage"
  | "visuals"
  | "gameSpecific";

export type RiskLevel = "low" | "medium" | "high";
export type ImpactLevel = "low" | "medium" | "high";
export type DetectedState = "on" | "off" | "unknown" | "notApplicable";

export interface ScoreFactor {
  label: string;
  points: number;
  maxPoints: number;
  note: string;
}

export interface SystemReport {
  cpu: {
    vendor: string;
    model: string;
    cores: number;
    logicalProcessors: number;
    generationHint: string;
    baseClockMhz?: number;
  };
  gpu: {
    vendor: string;
    model: string;
    vramMb?: number;
    driverVersion?: string;
  };
  ram: {
    totalGb: number;
    speedMhz?: number;
    ratedSpeedMhz?: number;
    xmpProfileDetected: boolean;
    runningBelowRated: boolean;
  };
  storage: Array<{
    model: string;
    kind: string;
    sizeGb: number;
    freeGb?: number;
  }>;
  motherboard: {
    manufacturer: string;
    model: string;
    biosVersion?: string;
    biosVendor?: string;
  };
  monitors: Array<{
    name: string;
    refreshHz?: number;
    resolution?: string;
  }>;
  windows: {
    version: string;
    build: string;
    edition: string;
  };
  performanceScore: number;
  scoreBreakdown: ScoreFactor[];
  scannedAt: string;
  isDemoData: boolean;
  scanMethod: string;
  scanWarnings: string[];
}

export interface TweakDefinition {
  id: string;
  name: string;
  description: string;
  expectedBenefit: string;
  category: TweakCategory;
  risk: RiskLevel;
  impact: ImpactLevel;
  requiresReboot: boolean;
  profiles: OptimizationProfile[];
  guidanceOnly: boolean;
}

export interface TweakState {
  id: string;
  applied: boolean;
  appliedAt?: string;
  sessionId?: string;
}

export interface TweakCatalogEntry {
  definition: TweakDefinition;
  state: TweakState;
  detected: DetectedState;
  enabled: boolean;
}

export interface ApplyProfileResult {
  sessionId: string;
  applied: string[];
  skipped: string[];
  guidance: string[];
}

export interface CustomProfile {
  id: string;
  name: string;
  tweakIds: string[];
  createdAt: string;
  updatedAt: string;
}

export interface BenchmarkHistoryEntry {
  id: string;
  capturedAt: string;
  phase: "baseline" | "afterTweaks" | "afterBios" | "custom";
  label: string;
  source: "cs2_live" | "cs2_microbench" | "cs2_estimate" | string;
  frameSampleCount?: number;
  captureDurationSecs?: number;
  runId?: string;
  cs2AvgFps: number;
  cs2FpsP1: number;
  cs2FpsP01: number;
  frametimeJitterMs: number;
  systemLatencyScore: number;
  timerLatencyUs: number;
  inputPathScore: number;
  cpuSingleThreadScore: number;
  memoryLatencyUs: number;
  activeTweakIds: string[];
  biosSummary: string;
  details: {
    cpuSingle: number;
    cpuMulti: number;
    memoryBandwidth: number;
    storageReadMbps: number;
    storageWriteMbps: number;
  };
}

export interface BenchmarkPipelineLog {
  runId: string;
  startedAt: string;
  finishedAt?: string;
  source: string;
  steps: string[];
}

export interface Cs2CompetitiveResult {
  entry: BenchmarkHistoryEntry;
  history: { entries: BenchmarkHistoryEntry[] };
  deltaAvgFpsVsPrevious?: number;
  deltaP1FpsVsPrevious?: number;
  deltaAvgFpsVsStarting?: number;
  deltaP1FpsVsStarting?: number;
  summary: string;
  pipelineLog: BenchmarkPipelineLog;
}

export interface InteractiveBiosSetting {
  id: string;
  title: string;
  explanation: string;
  competitiveFpsNote: string;
  detectedStatus: string;
  detectedDetail: string;
  recommendedValue: string;
  menuPath: string;
  risk: string;
  requiresReboot: boolean;
  verifiedAfterReboot: boolean;
}

export interface InteractiveBiosReport {
  vendor: string;
  motherboard: string;
  settings: InteractiveBiosSetting[];
  rebootToFirmwareCommand: string;
  postRebootVerificationPending: boolean;
}

export interface SafeModeSession {
  id: string;
  createdAt: string;
  baselineScore: number;
  baselineAvgFps?: number;
  baselineP1Fps?: number;
  tweakIds: string[];
  confirmed: boolean;
}

export interface InstalledGame {
  appId: string;
  name: string;
  installDir: string;
  platform: string;
}

export interface Cs2Optimization {
  launchOptions: string;
  autoexec: string;
  videoSettings: Record<string, unknown>;
  notes: string[];
}

export interface BiosAssistantReport {
  vendor: string;
  motherboard: string;
  cpuVendor: string;
  items: Array<{
    id: string;
    title: string;
    recommendedValue: string;
    menuPath: string;
    why: string;
    risk: string;
  }>;
  rebootToFirmwareCommand: string;
}

export interface ElevationStatus {
  isWindows: boolean;
  isElevated: boolean;
  message: string;
}

export const CATEGORY_LABELS: Record<TweakCategory, string> = {
  powerCpu: "Power & CPU",
  gpu: "GPU",
  latencyTimers: "Latency & Timers",
  network: "Network",
  input: "Input",
  servicesBloat: "Services & bloat",
  memory: "Memory",
  storage: "Storage",
  visuals: "Visuals",
  gameSpecific: "Game-specific",
};
