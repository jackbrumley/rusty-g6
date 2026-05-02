export type AppRoute = "status" | "output" | "input" | "debug" | "ui-lab";

export const DEFAULT_ROUTE: AppRoute = "status";

export const routeFromHash = (hash: string): AppRoute => {
  const normalized = hash.replace(/^#\/?/, "").split("/")[0].trim().toLowerCase();
  if (
    normalized === "status" ||
    normalized === "output" ||
    normalized === "input" ||
    normalized === "debug" ||
    normalized === "ui-lab"
  ) {
    return normalized as AppRoute;
  }
  if (normalized === "main") {
    return "status";
  }
  if (normalized === "microphone") {
    return "input";
  }
  return DEFAULT_ROUTE;
};
