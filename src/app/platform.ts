/// Whether the webview runs on Linux. Linux-only features (USB permissions,
/// the recording source) check it.
export const IS_LINUX = navigator.userAgent.toLowerCase().includes("linux");
