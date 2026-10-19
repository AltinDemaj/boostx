import { useEffect, useState } from "react";

export type ToastKind = "success" | "error" | "info";

export function ToastStack({
  message,
  kind,
  onClear,
}: {
  message: string | null;
  kind: ToastKind;
  onClear: () => void;
}) {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (!message) {
      setVisible(false);
      return;
    }
    setVisible(true);
    const t = setTimeout(() => {
      setVisible(false);
      onClear();
    }, 4500);
    return () => clearTimeout(t);
  }, [message, onClear]);

  if (!message || !visible) return null;

  const colors =
    kind === "success"
      ? "border-teal-400/40 bg-teal-500/15 text-teal-50"
      : kind === "error"
        ? "border-rose-400/40 bg-rose-500/15 text-rose-50"
        : "border-slate-400/30 bg-slate-800/90 text-slate-100";

  return (
    <div
      className={`fixed bottom-6 right-6 z-50 max-w-md px-4 py-3 rounded-xl border shadow-xl text-sm ${colors}`}
      role="status"
    >
      {message}
    </div>
  );
}
