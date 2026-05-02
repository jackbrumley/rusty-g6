import type { ToastMessage } from "../../types/g6";

interface ToastHostProps {
  toast: ToastMessage | null;
  onDismiss: () => void;
  onPause: () => void;
  onResume: () => void;
}

export function ToastHost({ toast, onDismiss, onPause, onResume }: ToastHostProps) {
  return (
    <div class="toast-host" data-no-drag="true">
      {toast && (
        <div class={`toast toast-${toast.type}`} onMouseEnter={onPause} onMouseLeave={onResume}>
          <div class="toast-content">
            <p class="toast-message">{toast.message}</p>
            <button class="toast-close" onClick={onDismiss}>
              ×
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
