import { writable } from "svelte/store";
import {
  getActiveRuns,
  RUN_CANCEL_REQUESTED_EVENT,
  RUN_STARTED_EVENT,
  subscribeToEvents,
  type AnalysisRun,
  type RunCancelRequestedDetail,
  type RunEventStream,
  type RunStartedDetail,
  type RunEvent,
} from "$lib/api";

type ActiveRunsState = {
  runs: AnalysisRun[];
  loading: boolean;
  bootstrapped: boolean;
  error: string;
};

const state = writable<ActiveRunsState>({
  runs: [],
  loading: false,
  bootstrapped: false,
  error: "",
});

const streams = new Map<string, RunEventStream>();
let started = false;
let refreshInFlight = false;
let refreshQueued = false;
let refreshTimer: ReturnType<typeof setInterval> | null = null;
let detachRunStartedListener: (() => void) | null = null;
let detachCancelRequestedListener: (() => void) | null = null;

function isActiveRun(run: AnalysisRun): boolean {
  if (!(run.status === "pending" || run.status === "running")) {
    return false;
  }

  const stage = run.current_stage ?? "";
  return stage !== "cancelling" && stage !== "cancelled";
}

function sortRuns(runs: AnalysisRun[]): AnalysisRun[] {
  return [...runs].sort((a, b) => {
    const aTs = Date.parse(a.created_at ?? "");
    const bTs = Date.parse(b.created_at ?? "");
    const aVal = Number.isFinite(aTs) ? aTs : 0;
    const bVal = Number.isFinite(bTs) ? bTs : 0;
    return bVal - aVal;
  });
}

function removeRun(runId: string): void {
  state.update((current) => ({
    ...current,
    runs: current.runs.filter((run) => run.id !== runId),
  }));

  const existing = streams.get(runId);
  if (existing) {
    existing.close();
    streams.delete(runId);
  }
}

function upsertOptimisticRun(detail: RunStartedDetail): void {
  const optimisticRun: AnalysisRun = {
    id: detail.runId,
    url: detail.url,
    analysis_type: detail.analysisType,
    status: "running",
    progress: 0,
    current_stage: "queued",
    current_message: "Starting run...",
    created_at: new Date().toISOString(),
  };

  state.update((current) => {
    const deduped = current.runs.filter((run) => run.id !== detail.runId);
    return {
      ...current,
      runs: sortRuns([optimisticRun, ...deduped]),
      error: "",
      bootstrapped: true,
    };
  });

  reconcileStreams([optimisticRun, ...getCurrentRunsWithout(detail.runId)]);
}

function getCurrentRunsWithout(runId: string): AnalysisRun[] {
  let snapshot: AnalysisRun[] = [];
  state.update((current) => {
    snapshot = current.runs.filter((run) => run.id !== runId);
    return current;
  });
  return snapshot;
}

function registerGlobalRunLifecycleListeners(): void {
  if (typeof window === "undefined") return;
  if (!detachRunStartedListener) {
    const startedHandler = (event: Event) => {
      const detail = (event as CustomEvent<RunStartedDetail>).detail;
      if (!detail?.runId) return;
      upsertOptimisticRun(detail);
      void refresh();
    };
    window.addEventListener(RUN_STARTED_EVENT, startedHandler);
    detachRunStartedListener = () => {
      window.removeEventListener(RUN_STARTED_EVENT, startedHandler);
    };
  }

  if (!detachCancelRequestedListener) {
    const cancelHandler = (event: Event) => {
      const detail = (event as CustomEvent<RunCancelRequestedDetail>).detail;
      if (!detail?.runId) return;
      removeRun(detail.runId);
      void refresh();
    };
    window.addEventListener(RUN_CANCEL_REQUESTED_EVENT, cancelHandler);
    detachCancelRequestedListener = () => {
      window.removeEventListener(RUN_CANCEL_REQUESTED_EVENT, cancelHandler);
    };
  }
}

function unregisterGlobalRunLifecycleListeners(): void {
  if (detachRunStartedListener) {
    detachRunStartedListener();
    detachRunStartedListener = null;
  }
  if (detachCancelRequestedListener) {
    detachCancelRequestedListener();
    detachCancelRequestedListener = null;
  }
}

function reconcileStreams(runs: AnalysisRun[]): void {
  const activeRunIds = new Set(runs.map((run) => run.id));

  for (const run of runs) {
    if (!streams.has(run.id)) {
      const stream = subscribeToEvents(
        run.id,
        (event) => handleRunEvent(event),
        () => {
          const existing = streams.get(run.id);
          if (existing) {
            existing.close();
            streams.delete(run.id);
          }
          void refresh();
        },
      );
      streams.set(run.id, stream);
    }
  }

  for (const [runId, stream] of streams) {
    if (!activeRunIds.has(runId)) {
      stream.close();
      streams.delete(runId);
    }
  }
}

function applyRunEvent(current: AnalysisRun, event: RunEvent): AnalysisRun {
  if (event.kind === "progress") {
    return {
      ...current,
      status: "running",
      current_stage: event.stage ?? current.current_stage,
      current_message: event.message ?? current.current_message,
      progress: event.progress ?? current.progress,
    };
  }

  if (event.kind === "failed") {
    return {
      ...current,
      status: "failed",
      current_stage: event.stage ?? current.current_stage,
      current_message: event.message ?? current.current_message,
      progress: event.progress ?? current.progress,
    };
  }

  if (event.kind === "cancelled") {
    return {
      ...current,
      status: "failed",
      current_stage: event.stage ?? "cancelled",
      current_message: event.message ?? current.current_message,
      progress: event.progress ?? current.progress,
    };
  }

  if (event.kind === "complete") {
    return {
      ...current,
      status: "completed",
      current_stage: event.stage ?? "complete",
      current_message: event.message ?? current.current_message,
      progress: event.progress ?? 1,
    };
  }

  return current;
}

function handleRunEvent(event: RunEvent): void {
  state.update((currentState) => {
    const runs = currentState.runs
      .map((run) => {
        if (run.id !== event.run_id) return run;
        return applyRunEvent(run, event);
      })
      .filter((run) => isActiveRun(run));

    return {
      ...currentState,
      runs: sortRuns(runs),
    };
  });

  if (
    event.kind === "complete" ||
    event.kind === "failed" ||
    event.kind === "cancelled"
  ) {
    const stream = streams.get(event.run_id);
    if (stream) {
      stream.close();
      streams.delete(event.run_id);
    }
    void refresh();
  }
}

async function refresh(): Promise<void> {
  if (refreshInFlight) {
    refreshQueued = true;
    return;
  }
  refreshInFlight = true;
  state.update((current) => ({ ...current, loading: true }));

  try {
    const runs = sortRuns(await getActiveRuns()).filter((run) =>
      isActiveRun(run),
    );

    state.update((current) => ({
      ...current,
      runs,
      loading: false,
      bootstrapped: true,
      error: "",
    }));

    reconcileStreams(runs);
  } catch (err) {
    state.update((current) => ({
      ...current,
      loading: false,
      bootstrapped: true,
      error: err instanceof Error ? err.message : "Failed to load active runs",
    }));
  } finally {
    refreshInFlight = false;
    if (refreshQueued) {
      refreshQueued = false;
      void refresh();
    }
  }
}

function start(): void {
  if (started) return;
  started = true;
  registerGlobalRunLifecycleListeners();
  void refresh();
  refreshTimer = setInterval(() => {
    void refresh();
  }, 4000);
}

function stop(): void {
  if (!started) return;
  started = false;
  unregisterGlobalRunLifecycleListeners();
  if (refreshTimer) {
    clearInterval(refreshTimer);
    refreshTimer = null;
  }

  for (const [, stream] of streams) {
    stream.close();
  }
  streams.clear();
}

export const activeRunsStore = {
  subscribe: state.subscribe,
  start,
  stop,
  refresh,
};
