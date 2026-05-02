import { invoke } from "@tauri-apps/api/core";
import { useEffect } from "preact/hooks";

const normalizeText = (value: string | null | undefined) => {
  if (!value) {
    return "";
  }
  return value.replace(/\s+/g, " ").trim();
};

const getControlLabel = (element: HTMLElement) => {
  const controlRow = element.closest(".control-row");
  const label = controlRow?.querySelector(".control-label");
  const text = normalizeText(label?.textContent);
  return text || "Unlabeled control";
};

export function useUiInteractionLogger() {
  useEffect(() => {
    const logEvent = (message: string) => {
      invoke("log_ui_event", { message }).catch(() => {
        // no-op
      });
    };

    const onClickCapture = (event: MouseEvent) => {
      const target = event.target as HTMLElement | null;
      const button = target?.closest("button");
      if (!button) {
        return;
      }

      const label =
        normalizeText(button.textContent) ||
        normalizeText(button.getAttribute("aria-label")) ||
        normalizeText(button.getAttribute("title")) ||
        "Unnamed button";

      logEvent(`Button clicked: ${label}`);
    };

    const onChangeCapture = (event: Event) => {
      const target = event.target as HTMLInputElement | null;
      if (!target || target.tagName !== "INPUT") {
        return;
      }

      if (target.type === "checkbox") {
        logEvent(`Toggle changed: ${getControlLabel(target)} -> ${target.checked ? "On" : "Off"}`);
        return;
      }

      if (target.type === "range") {
        logEvent(`Slider changed: ${getControlLabel(target)} -> ${target.value}`);
      }
    };

    window.addEventListener("click", onClickCapture, true);
    window.addEventListener("change", onChangeCapture, true);

    return () => {
      window.removeEventListener("click", onClickCapture, true);
      window.removeEventListener("change", onChangeCapture, true);
    };
  }, []);
}
