// SPDX-License-Identifier: MIT OR Apache-2.0

import { createHash } from 'node:crypto'
import { readdir, readFile } from 'node:fs/promises'

import { svelte } from '@sveltejs/vite-plugin-svelte'
import { playwright } from '@vitest/browser-playwright'
import { defineConfig, type Plugin } from 'vitest/config'

const SERVICE_WORKER = new URL('src/offline/sw.js', import.meta.url)
const PUBLIC = new URL('public/', import.meta.url)
const FONTS = new URL('node_modules/@fontsource-variable/', import.meta.url)

/** The typefaces the app ships, by package, and the name their licence is written as. */
const TYPEFACES = [
	['recursive', 'Recursive'],
	['ibm-plex-sans', 'IBM-Plex-Sans'],
] as const

/**
 * Writes each bundled typeface's licence into `dist/fonts/` (ADR 0015).
 *
 * The OFL lets a font travel with any software, on condition that its
 * copyright and licence travel with it. Vite copies the font files out of
 * the packages and leaves their `LICENSE` behind, so this puts it back.
 *
 * Before the offline shell in the plugin list, so the licences land in the
 * precache with everything else.
 */
function fontLicences(): Plugin {
	return {
		name: 'pastille-font-licences',
		apply: 'build',
		async generateBundle() {
			const names = []
			for (const [folder, name] of TYPEFACES) {
				const source = await readFile(new URL(`${folder}/LICENSE`, FONTS), 'utf8')
				this.emitFile({ type: 'asset', fileName: `fonts/${name}-LICENSE.txt`, source })
				names.push(`- ${name}-LICENSE.txt`)
			}
			this.emitFile({
				type: 'asset',
				fileName: 'fonts/README.txt',
				source: [
					'The typefaces in this app are used unmodified under the SIL Open Font',
					'License 1.1; the app itself is MIT OR Apache-2.0. Their licences:',
					'',
					...names,
					'',
				].join('\n'),
			})
		},
	}
}

const PROGRAMS = new URL('../programs/', import.meta.url)

/** Where the source is, for whoever holds only the built app. */
const REPOSITORY = 'https://github.com/Srynetix/pastille'

/**
 * Writes the Uno programs' notice and the LGPL into `dist/programs/`.
 *
 * The example circuits carry compiled Arduino programs, and each one links
 * Arduino's AVR core, which is LGPL-2.1 or later. The licence asks that its
 * text and the way to rebuild the program travel with it, in the built app as
 * much as in the repository: `programs/NOTICE.md` says what is inside, and
 * the repository holds the sketches and the build script.
 *
 * Before the offline shell, like the font licences, so the precache has them.
 */
function programLicences(): Plugin {
	return {
		name: 'pastille-program-licences',
		apply: 'build',
		async generateBundle() {
			for (const name of ['NOTICE.md', 'LGPL-2.1.txt']) {
				const source = await readFile(new URL(name, PROGRAMS), 'utf8')
				this.emitFile({ type: 'asset', fileName: `programs/${name}`, source })
			}
			this.emitFile({
				type: 'asset',
				fileName: 'programs/README.txt',
				source: [
					'The example circuits with an Arduino Uno carry compiled programs that',
					"contain Arduino's AVR core, under the GNU LGPL 2.1 or later; the app",
					'itself is MIT OR Apache-2.0. What each program contains and how to',
					'rebuild it are in NOTICE.md, the licence in LGPL-2.1.txt, and the',
					`sketches and the build script in the source repository, ${REPOSITORY}.`,
					'',
				].join('\n'),
			})
		},
	}
}

/**
 * Emits the service worker, with the list of files this build produced.
 *
 * C-27. The worker itself is `src/offline/sw.js` and is not bundled — see the
 * comment at the top of it. All this does is fill in the two things it cannot
 * know: what to cache, and which build it belongs to.
 *
 * Build only. A service worker in the dev server would serve yesterday's app
 * back over the top of the one being edited, which is the classic way to lose
 * an afternoon.
 */
function offlineShell(): Plugin {
	return {
		name: 'pastille-offline-shell',
		apply: 'build',
		async generateBundle(_options, bundle) {
			// Two sources, because Vite only knows about one of them: the
			// compiled chunks and assets it emitted, and the files it copies
			// verbatim out of `public/` — the manifest and the icons an install
			// needs, which are exactly the files an installed app asks for first.
			const built = Object.keys(bundle)
			const copied = await readdir(PUBLIC)

			// The document is named rather than found: Vite writes it in a
			// later hook than this one, and it is the file the whole thing
			// turns on — the one URL that never changes, and the one every
			// navigation is answered with.
			// Relative, so the worker caches them wherever the app is served
			// from: the root of a server, or under GitHub Pages' /pastille/.
			const precache = ['index.html', ...built, ...copied].sort()

			// The build's fingerprint is the list itself. Every asset but the
			// document is content-hashed in its name already, so a list that
			// has not changed is a build that produces the same bytes, and one
			// that has is a new cache and a dropped old one.
			const version = createHash('sha256').update(precache.join('\n')).digest('hex').slice(0, 12)

			const source = (await readFile(SERVICE_WORKER, 'utf8'))
				.replace('__PRECACHE__', JSON.stringify(precache, null, '\t'))
				.replace('__VERSION__', version)

			this.emitFile({ type: 'asset', fileName: 'sw.js', source })
		},
	}
}

/**
 * Starts fetching the wasm core from the document, rather than from the code
 * that needs it (M-21).
 *
 * Without this the core is a waterfall: the browser downloads the script,
 * runs it, and only then learns there is a 300 KB bundle to fetch. On a slow
 * phone (`just coldload`'s slow 4G with a 4x slower CPU) that put the request
 * 1.3 s in and the core ready at 4.7 s; preloaded, it is ready at 3.7 s.
 *
 * The price is paid by the shell, which shares the line with the core now and
 * is drawn half a second later. Taken knowingly: until the core is there the
 * shell can only say it is loading, and a beginner is waiting for a circuit
 * that works, not for a header.
 *
 * `as="fetch"` with `crossorigin` is what the glue's own `fetch` asks for, so
 * the browser hands that one the preloaded response rather than downloading
 * the file twice. Build only: in dev the file is neither hashed nor slow.
 */
function preloadCore(): Plugin {
	let base = '/'
	return {
		name: 'pastille-preload-core',
		apply: 'build',
		configResolved(config) {
			base = config.base
		},
		transformIndexHtml: {
			order: 'post',
			handler(_html, context) {
				const core = Object.keys(context.bundle ?? {}).find((file) => file.endsWith('.wasm'))
				if (!core) return []
				return [
					{
						tag: 'link',
						attrs: {
							rel: 'preload',
							href: `${base}${core}`,
							as: 'fetch',
							type: 'application/wasm',
							crossorigin: '',
						},
						injectTo: 'head',
					},
				]
			},
		},
	}
}

export default defineConfig({
	// Where the app is served from: the root of a server unless the build
	// says otherwise, as `just pages` does for GitHub Pages' /pastille/
	// (ADR 0021). The desktop shell loads the root build.
	base: process.env.PASTILLE_BASE ?? '/',
	plugins: [svelte(), fontLicences(), programLicences(), offlineShell(), preloadCore()],
	test: {
		// The wasm boundary can only be exercised honestly in a browser: Node's
		// module loader and memory model are not the target (M-02).
		browser: {
			enabled: true,
			headless: true,
			provider: playwright(),
			instances: [{ browser: 'chromium' }],
		},
	},
})
