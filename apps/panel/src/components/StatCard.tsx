import type { Component } from "solid-js";
import { Show } from "solid-js";
import { Dynamic } from "solid-js/web";

interface StatCardProps {
  label: string;
  value: string;
  hint?: string;
  tone?: "default" | "success" | "warning" | "danger" | "muted";
  icon?: Component<{ class?: string }>;
}

export function StatCard(props: StatCardProps) {
  const tone = () => props.tone ?? "default";
  return (
    <div class={`stat-card stat-${tone()}`}>
      <Show when={props.icon} keyed>
        {Icon => (
          <span class="stat-icon">
            <Dynamic component={Icon} />
          </span>
        )}
      </Show>
      <div class="stat-body">
        <div class="stat-label">{props.label}</div>
        <div class="stat-value">{props.value}</div>
        <Show when={props.hint}>
          <div class="stat-hint">{props.hint}</div>
        </Show>
      </div>
    </div>
  );
}
