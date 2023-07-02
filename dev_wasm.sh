
wasm-pack build --dev --target web --out-dir server/pkg
# rm -rf server/pkg
# mv pkg server/
cd server
npm run start
