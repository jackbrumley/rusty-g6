import { useEffect, useState } from "preact/hooks";
import { routeFromHash, type AppRoute } from "../app/routes";

/// The active page, kept in the URL hash.
export function useRoute() {
  const [activeTab, setActiveTab] = useState<AppRoute>(routeFromHash(window.location.hash));

  useEffect(() => {
    const syncRouteFromHash = () => setActiveTab(routeFromHash(window.location.hash));
    window.addEventListener("hashchange", syncRouteFromHash);
    syncRouteFromHash();
    return () => window.removeEventListener("hashchange", syncRouteFromHash);
  }, []);

  const navigate = (route: AppRoute) => {
    const nextHash = `#/${route}`;
    if (window.location.hash === nextHash) {
      setActiveTab(route);
      return;
    }
    window.location.hash = nextHash;
  };

  return { activeTab, navigate };
}
