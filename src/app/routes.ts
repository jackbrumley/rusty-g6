export type AppRoute = "status" | "sound" | "eq" | "mic" | "device" | "debug" | "ui-lab";

export const DEFAULT_ROUTE: AppRoute = "status";

const ROUTES: AppRoute[] = ["status", "sound", "eq", "mic", "device", "debug", "ui-lab"];

/// Earlier page names, so old links still land on the right page.
const RENAMED_ROUTES: Record<string, AppRoute> = {
  main: "status",
  output: "sound",
  input: "mic",
  microphone: "mic",
};

export const routeFromHash = (hash: string): AppRoute => {
  const name = hash.replace(/^#\/?/, "").split("/")[0].trim().toLowerCase();
  return ROUTES.find((route) => route === name) ?? RENAMED_ROUTES[name] ?? DEFAULT_ROUTE;
};
