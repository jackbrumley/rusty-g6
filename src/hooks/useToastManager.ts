import { useEffect, useRef, useState } from "preact/hooks";
import type { ToastMessage, ToastType } from "../types/g6";

export function useToastManager() {
  const [toast, setToast] = useState<ToastMessage | null>(null);
  const toastIdRef = useRef(0);
  const timerRef = useRef<number | null>(null);
  const startedAtRef = useRef(0);
  const remainingRef = useRef(0);

  const clearTimer = () => {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  };

  const dismissToast = () => {
    clearTimer();
    setToast(null);
  };

  const startTimer = () => {
    if (!toast) {
      return;
    }
    clearTimer();
    startedAtRef.current = Date.now();
    timerRef.current = window.setTimeout(() => {
      dismissToast();
    }, remainingRef.current);
  };

  const showToast = (
    message: string,
    type: ToastType,
    durationMs = type === "success" ? 3000 : 5000
  ) => {
    toastIdRef.current += 1;
    remainingRef.current = durationMs;
    setToast({
      id: toastIdRef.current,
      message,
      type,
      durationMs,
    });
  };

  const pauseToast = () => {
    if (!toast) {
      return;
    }
    const elapsed = Date.now() - startedAtRef.current;
    remainingRef.current = Math.max(0, remainingRef.current - elapsed);
    clearTimer();
  };

  const resumeToast = () => {
    if (!toast || remainingRef.current <= 0) {
      return;
    }
    startTimer();
  };

  useEffect(() => {
    if (!toast) {
      clearTimer();
      return;
    }
    remainingRef.current = toast.durationMs;
    startTimer();
    return () => {
      clearTimer();
    };
  }, [toast?.id]);

  return {
    toast,
    showToast,
    dismissToast,
    pauseToast,
    resumeToast,
  };
}
