import * as fs from 'fs'
import * as child_process from 'child_process'

if (process.argv[2] !== 'node' && process.argv[2] !== 'browser') {
    throw new Error("must be 'node' or 'browser'")
}
fs.writeFileSync('./pkg/borker_rs.d.ts', fs.readFileSync('./types/borker_rs.d.ts'))
pkg = require('./pkg/package.json')
pkg.name += '-' + process.argv[2]
fs.writeFileSync('./pkg/package.json', JSON.stringify(pkg, null, 2))
child_process.execSync('cd pkg && npm publish')