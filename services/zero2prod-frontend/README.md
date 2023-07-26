# Vue 3 + TypeScript + Vite

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) (and disable Vetur) + [TypeScript Vue Plugin (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.vscode-typescript-vue-plugin).

## Type Support For `.vue` Imports in TS

TypeScript cannot handle type information for `.vue` imports by default, so we replace the `tsc` CLI with `vue-tsc` for type checking. In editors, we need [TypeScript Vue Plugin (Volar)](https://marketplace.visualstudio.com/items?itemName=Vue.vscode-typescript-vue-plugin) to make the TypeScript language service aware of `.vue` types.

If the standalone TypeScript plugin doesn't feel fast enough to you, Volar has also implemented a [Take Over Mode](https://github.com/johnsoncodehk/volar/discussions/471#discussioncomment-1361669) that is more performant. You can enable it by the following steps:

1. Disable the built-in TypeScript Extension
   1. Run `Extensions: Show Built-in Extensions` from VSCode's command palette
   2. Find `TypeScript and JavaScript Language Features`, right click and select `Disable (Workspace)`
2. Reload the VSCode window by running `Developer: Reload Window` from the command palette.


## Bootstrapping

npm create vite@latest
cd eval-v1
npm install

nvim Dockerfile
docker build . -t a403/eval-v1
docker run -p 49160:8080 -d a403/eval-v1:latest
docker ps --all
docker logs -t elated_lumiere
docker stop elated_lumiere
docker rm elated_lumiere

npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
nvim tailwind.config.js
nvim postcss.config.js
nvim src/style.css
ls src/assets/
[copy fonts to src/assets/fonts]
npm run dev
git init
git add .
git commit -m "Baseline"

npm install --save-dev @vue/eslint-config-typescript @rushstack/eslint-patch
npm install --save-dev eslint eslint-plugin-vue
npm install --save-dev eslint-config-prettier
npm install --save-dev prettier
cp ../vue-enterprise-boilerplate/.prettierrc.json .
cp ../vue-enterprise-boilerplate/.eslintrc.cjs .
nvim package.json
nvim .eslintrc.cjs
npm run lint
git add .
git commit -m "Add eslint"
