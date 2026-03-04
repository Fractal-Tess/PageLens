export type RunKind = 'single' | 'crawl' | 'site_files'

export type SingleOptions = {
  includeHtml: boolean
  includeAccessibilityTree: boolean
  includePerformanceTiming: boolean
  includeComputedStyles: boolean
}

export type CrawlOptions = {
  maxPages: number
  maxDepth: number
  followExternalLinks: boolean
  sameSubdomainOnly: boolean
  pageTimeoutMs: number
  delayMs: number
  maxConcurrency: number
}

export type AnalysisProfile = {
  id: string
  name: string
  description: string
  runKind: RunKind
  singleOptions: SingleOptions
  crawlOptions: CrawlOptions
  createdAt: string
  updatedAt: string
}

export const DEFAULT_SINGLE_OPTIONS: SingleOptions = {
  includeHtml: true,
  includeAccessibilityTree: true,
  includePerformanceTiming: true,
  includeComputedStyles: false
}

export const DEFAULT_CRAWL_OPTIONS: CrawlOptions = {
  maxPages: 50,
  maxDepth: 3,
  followExternalLinks: false,
  sameSubdomainOnly: true,
  pageTimeoutMs: 30000,
  delayMs: 100,
  maxConcurrency: 4
}

const PROFILES_STORAGE_KEY = 'pagelens.analysis.profiles.v1'
const DEFAULT_PROFILE_STORAGE_KEY = 'pagelens.analysis.profiles.default.v1'

export function createBlankProfile(): AnalysisProfile {
  const now = new Date().toISOString()
  return {
    id: crypto.randomUUID(),
    name: 'New profile',
    description: '',
    runKind: 'crawl',
    singleOptions: { ...DEFAULT_SINGLE_OPTIONS },
    crawlOptions: { ...DEFAULT_CRAWL_OPTIONS },
    createdAt: now,
    updatedAt: now
  }
}

export function loadProfiles(): AnalysisProfile[] {
  if (typeof localStorage === 'undefined') return []
  const raw = localStorage.getItem(PROFILES_STORAGE_KEY)
  if (!raw) return []

  try {
    const parsed = JSON.parse(raw) as AnalysisProfile[]
    return parsed.map(profile => ({
      ...profile,
      singleOptions: {
        ...DEFAULT_SINGLE_OPTIONS,
        ...profile.singleOptions
      },
      crawlOptions: {
        ...DEFAULT_CRAWL_OPTIONS,
        ...profile.crawlOptions
      },
      description: profile.description ?? '',
      runKind: profile.runKind ?? 'crawl'
    }))
  } catch {
    return []
  }
}

export function saveProfiles(profiles: AnalysisProfile[]): void {
  if (typeof localStorage === 'undefined') return
  localStorage.setItem(PROFILES_STORAGE_KEY, JSON.stringify(profiles))
}

export function loadDefaultProfileId(): string | null {
  if (typeof localStorage === 'undefined') return null
  return localStorage.getItem(DEFAULT_PROFILE_STORAGE_KEY)
}

export function saveDefaultProfileId(profileId: string | null): void {
  if (typeof localStorage === 'undefined') return
  if (!profileId) {
    localStorage.removeItem(DEFAULT_PROFILE_STORAGE_KEY)
    return
  }
  localStorage.setItem(DEFAULT_PROFILE_STORAGE_KEY, profileId)
}
