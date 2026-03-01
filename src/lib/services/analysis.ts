import { invoke } from '@tauri-apps/api/core'
import type {
  AnalysisResult,
  AnalyzeUrlInput,
  CrawlUrlInput,
  HistoryListItem,
  AnalysisRun,
  UpdateAnalysisRun
} from '$lib/types'

/**
 * Analyze a single URL and store the result.
 */
export async function analyzeUrl(
  input: AnalyzeUrlInput
): Promise<AnalysisResult> {
  return await invoke('analyze_url', { input })
}

/**
 * Crawl a URL and store the result.
 */
export async function crawlUrl(input: CrawlUrlInput): Promise<AnalysisResult> {
  return await invoke('crawl_url', { input })
}

/**
 * List all analysis history items.
 */
export async function listHistory(
  limit?: number,
  offset?: number
): Promise<HistoryListItem[]> {
  return await invoke('list_history', { limit, offset })
}

/**
 * Get a single history item by ID.
 */
export async function getHistoryItem(id: string): Promise<AnalysisRun> {
  return await invoke('get_history_item', { id })
}

/**
 * Update a history item (e.g., rename).
 */
export async function updateHistoryItem(
  id: string,
  input: UpdateAnalysisRun
): Promise<AnalysisRun> {
  return await invoke('update_history_item', { id, input })
}

/**
 * Delete a single history item.
 */
export async function deleteHistoryItem(id: string): Promise<void> {
  return await invoke('delete_history_item', { id })
}

/**
 * Delete all history items.
 */
export async function deleteAllHistory(): Promise<number> {
  return await invoke('delete_all_history')
}

/**
 * Export history to a JSON file.
 */
export async function exportHistory(path: string): Promise<void> {
  return await invoke('export_history', { path })
}

/**
 * Import history from a JSON file.
 */
export async function importHistory(path: string): Promise<number> {
  return await invoke('import_history', { path })
}
