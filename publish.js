fs = require('fs')
child_process = require('child_process')

if (process.argv[2] === 'node') {
    child_process.execSync('wasm-pack build --target=nodejs', { stdio: "inherit" })
} else if (process.argv[2] === 'browser') {
    child_process.execSync('wasm-pack build --target=browser', { stdio: "inherit" })
} else {
    throw new Error("must be 'node' or 'browser'")
}
fs.writeFileSync('./pkg/borker_rs.d.ts', fs.readFileSync('./types/borker_rs.d.ts'))
fs.appendFileSync('./pkg/borker_rs.js', fs.readFileSync('./types/borker_enums_' + process.argv[2] + '.js'))
pkg = require('./pkg/package.json')
pkg.name += '-' + process.argv[2]
fs.writeFileSync('./pkg/package.json', JSON.stringify(pkg, null, 2))
child_process.execSync('cd pkg && npm publish', { stdio: "inherit" })