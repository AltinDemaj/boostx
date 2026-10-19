import { invoke } from "@tauri-apps/api/core";
import type { SystemReport } from "../types";

export type ToggleTweakResult = {
  enabled: boolean;
  verified: boolean;
  message: string;
};

export async function toggleTweak(
  tweakId: string,
  enable: boolean,
  report: SystemReport,
): Promise<ToggleTweakResult> {
  return invoke<ToggleTweakResult>("toggle_tweak_cmd", {
    tweakId,
    enable,
    report,
  });
}
