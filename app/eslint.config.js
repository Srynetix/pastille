// SPDX-License-Identifier: MIT OR Apache-2.0

// The app's linter (ADR 0019): the recommended rules of ESLint, of
// typescript-eslint with types, and of eslint-plugin-svelte. Nothing about
// layout — that is Prettier's (ADR 0018) — and nothing turned off without
// the reason beside it.

import js from '@eslint/js'
import svelte from 'eslint-plugin-svelte'
import globals from 'globals'
import ts from 'typescript-eslint'
import svelteConfig from './svelte.config.js'

export default ts.config(
	// The wasm bundle is generated (`just wasm`), and so is what Vite builds.
	{ ignores: ['dist/', 'src/lib/wasm/', '.vitest/'] },
	js.configs.recommended,
	...ts.configs.recommendedTypeChecked,
	...svelte.configs.recommended,
	{
		languageOptions: {
			globals: { ...globals.browser, ...globals.node },
			parserOptions: {
				// The two config files in JavaScript belong to no tsconfig.
				projectService: { allowDefaultProject: ['*.js'] },
				extraFileExtensions: ['.svelte'],
				tsconfigRootDir: import.meta.dirname,
			},
		},
		rules: {
			// `_` is the name for a value a pattern has to bind and nobody reads.
			'@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_', varsIgnorePattern: '^_' }],
		},
	},
	{
		files: ['**/*.svelte', '**/*.svelte.ts'],
		languageOptions: { parserOptions: { parser: ts.parser, svelteConfig } },
	},
	{
		// The linter's TypeScript cannot read a `.svelte` import's types, so in
		// a component everything that comes from a child component — its
		// exports, its callbacks' arguments — is `any` to it, and these rules
		// would flag every use. `svelte-check`, in the gate (`just types`),
		// does read them and fails on what these would catch. They stay on
		// in `.ts` files, where the types are whole.
		files: ['**/*.svelte'],
		rules: {
			'@typescript-eslint/no-unsafe-argument': 'off',
			'@typescript-eslint/no-unsafe-assignment': 'off',
			'@typescript-eslint/no-unsafe-call': 'off',
			'@typescript-eslint/no-unsafe-member-access': 'off',
			'@typescript-eslint/no-unsafe-return': 'off',
		},
	},
)
