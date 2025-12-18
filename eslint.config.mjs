import { defineConfig, globalIgnores } from "eslint/config";
import typescriptEslint from "@typescript-eslint/eslint-plugin";
import stylistic from "@stylistic/eslint-plugin";
import tsParser from "@typescript-eslint/parser";

export default defineConfig([
  globalIgnores(["**/out", "**/dist", "**/*.d.ts"]),
  {
    plugins: {
      "@typescript-eslint": typescriptEslint,
      "@stylistic": stylistic,
    },

    languageOptions: {
      parser: tsParser,
      ecmaVersion: 6,
      sourceType: "module",
    },

    rules: {
      "@typescript-eslint/naming-convention": "warn",
      "@stylistic/semi": "warn",
      curly: "warn",
      eqeqeq: "warn",
      "no-throw-literal": "warn",
      semi: "off",
    },
  },
]);
