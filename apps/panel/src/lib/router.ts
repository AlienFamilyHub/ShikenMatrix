import { ref } from "vue";

export type RouteName = "presence" | "control";

const ROUTE_PATHS: Record<RouteName, string> = {
  presence: "/",
  control: "/control",
};

export const route = ref<RouteName>(
  window.location.pathname.startsWith("/control")
    ? "control"
    : "presence",
);

export function navigate(next: RouteName): void {
  if (route.value !== next) {
    window.history.pushState({ route: next }, "", ROUTE_PATHS[next]);
    route.value = next;
    window.scrollTo({ top: 0 });
  }
}

export function hrefFor(next: RouteName): string {
  return ROUTE_PATHS[next];
}

export function startRouter(): void {
  window.addEventListener("popstate", () => {
    route.value = window.location.pathname.startsWith("/control")
      ? "control"
      : "presence";
  });
}
