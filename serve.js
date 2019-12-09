const express = require('express')
const cors = require('cors')
express.static.mime.types['wasm'] = 'application/wasm'
const app = express()
app.use(cors())
app.use(express.static('pkg'))
app.listen(8000, () => console.log('Serving at http://localhost:8000!'))
