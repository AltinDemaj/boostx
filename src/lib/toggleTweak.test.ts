import { describe, expect, it, vi, beforeEach } from "vitest";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import { toggleTweak } from "./toggleTweak";

describe("toggleTweak", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("calls toggle_tweak_cmd with camelCase args expected by Tauri", async () => {
    invokeMock.mockResolvedValue({ verified: true, message: "ok", enabled: true });
    const report = { performanceScore: 80, isDemoData: false } as never;

    await toggleTweak("power_min_processor_100", true, report);

    expect(invokeMock).toHaveBeenCalledWith("toggle_tweak_cmd", {
      tweakId: "power_min_processor_100",
      enable: true,
      report,
    });
  });
});
