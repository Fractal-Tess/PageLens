<script lang="ts">
	type Binding = { action: string; keys: string[] };
	type ModeSection = {
		id: string;
		name: string;
		trigger: string;
		color: string;
		glow: string;
		bindings: Binding[];
	};

	const modes: ModeSection[] = [
		{
			id: 'global',
			name: 'GLOBAL',
			trigger: 'always active',
			color: '#e8b84b',
			glow: 'rgba(232,184,75,0.35)',
			bindings: [
				{ action: 'Toggle Lock mode', keys: ['Ctrl+g'] },
				{ action: 'Quit Zellij', keys: ['Ctrl+q'] },
				{ action: 'Switch to Normal', keys: ['Esc'] },
				{ action: 'Confirm / Execute', keys: ['Enter'] }
			]
		},
		{
			id: 'pane',
			name: 'PANE',
			trigger: 'Ctrl+p',
			color: '#e8b84b',
			glow: 'rgba(232,184,75,0.35)',
			bindings: [
				{ action: 'Focus left', keys: ['h', '←'] },
				{ action: 'Focus right', keys: ['l', '→'] },
				{ action: 'Focus down', keys: ['j', '↓'] },
				{ action: 'Focus up', keys: ['k', '↑'] },
				{ action: 'Focus next pane', keys: ['p'] },
				{ action: 'New pane', keys: ['n'] },
				{ action: 'Split down', keys: ['d'] },
				{ action: 'Split right', keys: ['r'] },
				{ action: 'Close focused', keys: ['x'] },
				{ action: 'Toggle fullscreen', keys: ['f'] },
				{ action: 'Toggle pane frames', keys: ['z'] },
				{ action: 'Toggle floating', keys: ['w'] },
				{ action: 'Embed / float pane', keys: ['e'] },
				{ action: 'Rename pane', keys: ['c'] },
				{ action: 'Return to Normal', keys: ['Esc', 'Enter'] }
			]
		},
		{
			id: 'tab',
			name: 'TAB',
			trigger: 'Ctrl+t',
			color: '#38bdf8',
			glow: 'rgba(56,189,248,0.35)',
			bindings: [
				{ action: 'Previous tab', keys: ['h', '←'] },
				{ action: 'Next tab', keys: ['l', '→'] },
				{ action: 'New tab', keys: ['n'] },
				{ action: 'Close tab', keys: ['x'] },
				{ action: 'Rename tab', keys: ['r'] },
				{ action: 'Toggle sync tabs', keys: ['s'] },
				{ action: 'Break pane → new tab', keys: ['b'] },
				{ action: 'Break pane left', keys: ['['] },
				{ action: 'Break pane right', keys: [']'] },
				{ action: 'Go to tab 1–9', keys: ['1–9'] },
				{ action: 'Tab switcher', keys: ['Tab'] },
				{ action: 'Return to Normal', keys: ['Esc', 'Enter'] }
			]
		},
		{
			id: 'resize',
			name: 'RESIZE',
			trigger: 'Ctrl+n',
			color: '#a3e635',
			glow: 'rgba(163,230,53,0.35)',
			bindings: [
				{ action: 'Resize ←', keys: ['h', '←'] },
				{ action: 'Resize →', keys: ['l', '→'] },
				{ action: 'Resize ↓', keys: ['j', '↓'] },
				{ action: 'Resize ↑', keys: ['k', '↑'] },
				{ action: 'Expand left border', keys: ['H'] },
				{ action: 'Expand right border', keys: ['L'] },
				{ action: 'Expand bottom border', keys: ['J'] },
				{ action: 'Expand top border', keys: ['K'] },
				{ action: 'Increase size', keys: ['=', '+'] },
				{ action: 'Decrease size', keys: ['-'] },
				{ action: 'Return to Normal', keys: ['Esc', 'Enter'] }
			]
		},
		{
			id: 'move',
			name: 'MOVE',
			trigger: 'Ctrl+h',
			color: '#c084fc',
			glow: 'rgba(192,132,252,0.35)',
			bindings: [
				{ action: 'Move pane ←', keys: ['h', '←'] },
				{ action: 'Move pane →', keys: ['l', '→'] },
				{ action: 'Move pane ↓', keys: ['j', '↓'] },
				{ action: 'Move pane ↑', keys: ['k', '↑'] },
				{ action: 'Move to next position', keys: ['n'] },
				{ action: 'Move to prev position', keys: ['p'] },
				{ action: 'Return to Normal', keys: ['Esc', 'Enter'] }
			]
		},
		{
			id: 'scroll',
			name: 'SCROLL',
			trigger: 'Ctrl+s',
			color: '#fb923c',
			glow: 'rgba(251,146,60,0.35)',
			bindings: [
				{ action: 'Scroll down', keys: ['j', '↓'] },
				{ action: 'Scroll up', keys: ['k', '↑'] },
				{ action: 'Half page down', keys: ['d'] },
				{ action: 'Half page up', keys: ['u'] },
				{ action: 'Full page down', keys: ['Ctrl+f'] },
				{ action: 'Full page up', keys: ['Ctrl+b'] },
				{ action: 'Scroll to top', keys: ['g'] },
				{ action: 'Scroll to bottom', keys: ['G'] },
				{ action: 'Edit scrollback', keys: ['e'] },
				{ action: 'Enter Search', keys: ['s'] },
				{ action: 'Return to Normal', keys: ['Esc', 'Enter'] }
			]
		},
		{
			id: 'search',
			name: 'SEARCH',
			trigger: 'Ctrl+s → s',
			color: '#f87171',
			glow: 'rgba(248,113,113,0.35)',
			bindings: [
				{ action: 'Next match', keys: ['n'] },
				{ action: 'Previous match', keys: ['p'] },
				{ action: 'Toggle case sensitive', keys: ['c'] },
				{ action: 'Toggle wrap search', keys: ['w'] },
				{ action: 'Toggle whole word', keys: ['o'] },
				{ action: 'Return to Scroll', keys: ['Esc', 'Enter'] }
			]
		},
		{
			id: 'session',
			name: 'SESSION',
			trigger: 'Ctrl+o',
			color: '#2dd4bf',
			glow: 'rgba(45,212,191,0.35)',
			bindings: [
				{ action: 'Session manager', keys: ['w'] },
				{ action: 'Detach session', keys: ['d'] },
				{ action: 'Launch / focus plugin', keys: ['Ctrl+s'] },
				{ action: 'Return to Normal', keys: ['Esc', 'Enter'] }
			]
		},
		{
			id: 'tmux',
			name: 'TMUX',
			trigger: 'Ctrl+b (tmux compat)',
			color: '#818cf8',
			glow: 'rgba(129,140,248,0.35)',
			bindings: [
				{ action: 'Split vertical', keys: ['%'] },
				{ action: 'Split horizontal', keys: ['"'] },
				{ action: 'New window', keys: ['c'] },
				{ action: 'Next window', keys: ['n'] },
				{ action: 'Prev window', keys: ['p'] },
				{ action: 'Close pane', keys: ['x'] },
				{ action: 'Detach', keys: ['d'] },
				{ action: 'Rename window', keys: [','] },
				{ action: 'Window list', keys: ['w'] },
				{ action: 'Scroll mode', keys: ['['] }
			]
		}
	];
</script>

<svelte:head>
	<title>Zellij Keybindings · PageLens</title>
</svelte:head>

<div class="page-root">
	<!-- Header -->
	<div class="page-header">
		<div class="header-left">
			<div class="header-tag">TERMINAL MULTIPLEXER</div>
			<h1 class="header-title">ZELLIJ<span class="header-accent"> KEYBINDINGS</span></h1>
			<p class="header-sub">Default bindings · v0.40+ · All modes</p>
		</div>
		<div class="header-legend">
			<div class="legend-row">
				<span class="kbd-demo amber">Ctrl+p</span>
				<span class="legend-label">enter mode</span>
			</div>
			<div class="legend-row">
				<span class="kbd-demo muted">Esc</span>
				<span class="legend-label">exit mode</span>
			</div>
			<div class="legend-row">
				<span class="legend-slash">/</span>
				<span class="legend-label">alternative key</span>
			</div>
		</div>
	</div>

	<!-- Grid of mode sections -->
	<div class="modes-grid">
		{#each modes as mode (mode.id)}
			<div class="mode-card" style="--mode-color: {mode.color}; --mode-glow: {mode.glow}">
				<div class="mode-header">
					<span class="mode-name">{mode.name}</span>
					<span class="mode-trigger">{mode.trigger}</span>
				</div>
				<div class="bindings-list">
					{#each mode.bindings as binding (binding.action)}
						<div class="binding-row">
							<span class="binding-action">{binding.action}</span>
							<div class="binding-keys">
								{#each binding.keys as key, i (key)}
									{#if i > 0}<span class="key-sep">/</span>{/if}
									<kbd class="key-badge">{key}</kbd>
								{/each}
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/each}
	</div>

	<!-- Footer note -->
	<div class="page-footer-note">
		<span>Config at</span>
		<code>~/.config/zellij/config.kdl</code>
		<span>·</span>
		<span>Override with</span>
		<code>keybinds &#123; ... &#125;</code>
		<span>block</span>
	</div>
</div>

<style>
	.page-root {
		max-width: 1400px;
		margin: 0 auto;
		padding: 2rem 1.5rem 3rem;
	}

	/* ── Header ── */
	.page-header {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		gap: 2rem;
		margin-bottom: 2rem;
		padding-bottom: 1.25rem;
		border-bottom: 1px solid oklch(0.22 0 0);
	}

	.header-tag {
		font-size: 0.65rem;
		font-weight: 700;
		letter-spacing: 0.2em;
		color: oklch(0.72 0.19 47);
		margin-bottom: 0.35rem;
	}

	.header-title {
		font-size: clamp(1.5rem, 4vw, 2.75rem);
		font-weight: 900;
		letter-spacing: 0.05em;
		line-height: 1;
		color: oklch(0.96 0 0);
		margin: 0 0 0.4rem;
	}

	.header-accent {
		color: oklch(0.72 0.19 47);
	}

	.header-sub {
		font-size: 0.7rem;
		color: oklch(0.45 0 0);
		letter-spacing: 0.1em;
		margin: 0;
	}

	.header-legend {
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
		flex-shrink: 0;
	}

	.legend-row {
		display: flex;
		align-items: center;
		gap: 0.6rem;
	}

	.legend-label {
		font-size: 0.65rem;
		color: oklch(0.45 0 0);
		letter-spacing: 0.08em;
	}

	.legend-slash {
		font-size: 0.7rem;
		color: oklch(0.52 0 0);
		font-weight: 700;
		width: 2.5rem;
		text-align: center;
		padding: 0.15rem 0.5rem;
		border: 1px solid oklch(0.22 0 0);
		display: inline-block;
	}

	.kbd-demo {
		font-family: inherit;
		font-size: 0.65rem;
		font-weight: 700;
		padding: 0.15rem 0.5rem;
		border: 1px solid;
		letter-spacing: 0.05em;
	}

	.kbd-demo.amber {
		color: oklch(0.72 0.19 47);
		border-color: oklch(0.72 0.19 47);
		background: oklch(0.72 0.19 47 / 10%);
	}

	.kbd-demo.muted {
		color: oklch(0.52 0 0);
		border-color: oklch(0.3 0 0);
		background: transparent;
	}

	/* ── Grid ── */
	.modes-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: 1px;
		background: oklch(0.22 0 0);
		border: 1px solid oklch(0.22 0 0);
	}

	/* ── Mode Card ── */
	.mode-card {
		background: oklch(0.07 0 0);
		padding: 0;
		transition: background 0.15s ease;
	}

	.mode-card:hover {
		background: oklch(0.09 0 0);
	}

	.mode-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.5rem 0.85rem;
		border-bottom: 2px solid var(--mode-color);
		background: oklch(0.1 0 0);
	}

	.mode-name {
		font-size: 0.7rem;
		font-weight: 900;
		letter-spacing: 0.18em;
		color: var(--mode-color);
	}

	.mode-trigger {
		font-size: 0.6rem;
		color: oklch(0.45 0 0);
		letter-spacing: 0.08em;
		font-weight: 600;
	}

	/* ── Bindings ── */
	.bindings-list {
		padding: 0.3rem 0;
	}

	.binding-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 0.75rem;
		padding: 0.28rem 0.85rem;
		border-bottom: 1px solid oklch(0.12 0 0);
		transition: background 0.1s ease;
	}

	.binding-row:last-child {
		border-bottom: none;
	}

	.binding-row:hover {
		background: oklch(0.12 0 0);
	}

	.binding-action {
		font-size: 0.7rem;
		color: oklch(0.72 0 0);
		letter-spacing: 0.02em;
		flex: 1;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.binding-keys {
		display: flex;
		align-items: center;
		gap: 0.25rem;
		flex-shrink: 0;
	}

	.key-sep {
		font-size: 0.6rem;
		color: oklch(0.35 0 0);
	}

	kbd.key-badge {
		display: inline-block;
		font-family: inherit;
		font-size: 0.6rem;
		font-weight: 700;
		letter-spacing: 0.04em;
		padding: 0.1rem 0.4rem;
		border: 1px solid var(--mode-color, oklch(0.3 0 0));
		color: var(--mode-color);
		background: color-mix(in oklch, var(--mode-color) 8%, transparent);
		line-height: 1.4;
		white-space: nowrap;
		transition: background 0.15s ease, box-shadow 0.15s ease;
	}

	.binding-row:hover kbd.key-badge {
		background: color-mix(in oklch, var(--mode-color) 15%, transparent);
		box-shadow: 0 0 6px var(--mode-glow);
	}

	/* ── Footer note ── */
	.page-footer-note {
		margin-top: 1.5rem;
		padding-top: 1rem;
		border-top: 1px solid oklch(0.15 0 0);
		display: flex;
		align-items: center;
		gap: 0.5rem;
		flex-wrap: wrap;
		font-size: 0.65rem;
		color: oklch(0.38 0 0);
		letter-spacing: 0.05em;
	}

	.page-footer-note code {
		color: oklch(0.55 0 0);
		font-size: 0.65rem;
	}

	/* ── Responsive ── */
	@media (max-width: 640px) {
		.page-root {
			padding: 1.25rem 1rem 2rem;
		}

		.page-header {
			flex-direction: column;
			align-items: flex-start;
			gap: 1rem;
		}

		.header-legend {
			flex-direction: row;
			flex-wrap: wrap;
			gap: 0.75rem 1.5rem;
		}

		.modes-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
