module.exports = {
	"**/*.{js,ts,cjs,mjs,cts,mts,json,jsonc}": (files) => {
		// Filter out files in the .vscode folder.
		const filteredFiles = files.filter(
			(file) => !file.startsWith(".vscode/") && !file.includes("/.vscode/"),
		);

		// If no files remain after filtering, do nothing.
		if (filteredFiles.length === 0) return [];

		return [
			// Run biome format fix on the filtered files.
			`npx biome format --fix ${filteredFiles.join(" ")}`,
			// Run biome check write on the filtered files.
			`npx biome check --write ${filteredFiles.join(" ")}`,
		];
	},
};
