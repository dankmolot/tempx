const express = require('express')
const bodyParser = require('body-parser')
const mime = require('mime-types')
const duration = require('parse-duration')
const prettyms = require('pretty-ms')
const log = require('loglevel')
const fs = require('fs')
const path = require('path')
const uuid = require('./uuid')

const PORT = process.env.PORT || 8080
const STORE_PATH = path.resolve(process.env.STORE_PATH || './data')
const LOG_LEVEL = parseInt(process.env.LOG_LEVEL)
const UPLOAD_LIMIT = process.env.UPLOAD_LIMIT || '5mb'

if (!isNaN(LOG_LEVEL) && LOG_LEVEL != undefined) 
    log.setLevel(LOG_LEVEL)
else
    log.setLevel(2)

log.debug(`[DEBUG] Enviroment variables:\n\tPORT: ${PORT}\n\tSTORE_PATH: ${STORE_PATH}\n\tLOG_LEVEL: ${LOG_LEVEL}\n\tUPLOAD_LIMIT: ${UPLOAD_LIMIT}`)

log.info(`[INFO] Cleaning up ${STORE_PATH}`)
fs.rmdirSync(STORE_PATH, { recursive: true })
fs.mkdirSync(STORE_PATH, { recursive: true })

const app = express()
app.use(bodyParser.raw({
    type: '*/*',
    limit: UPLOAD_LIMIT
}))
app.use(function(req, res, next) { // My custom middleware
    log.debug(`[DEBUG] ${req.method} ${req.originalUrl} - ${req.ip} (${req.get('User-Agent')})`) // Logging every request

    res.error = function(err, status) {
        log.error(`[ERROR] ${req.method} ${req.originalUrl} - ${err}`)
        log.trace()
        return this.status(status || 500).json({ success: false, reason: err })
    }

    next()
})

function deleteFile(path, name, time) {
    setTimeout(() => {
        fs.rmSync(path)
        log.debug(`[DEBUG] File ${name} expired and removed.`)
    }, time)
}

app.post('/upload', (req, res) => {
    var expire = duration(req.query.expire || '1m') // time in minutes
    var name = uuid()
    var ext = mime.extension(req.get('Content-Type')) || 'bin'
    var filename = name + '.' + ext
    var filepath = path.join(STORE_PATH, filename)

    if (Object.keys(req.body).length === 0) // invalid body
        return res.error('Invalid body', 400)

    log.debug(`[DEBUG] Saving uploaded file to "${filepath}"...`)
    fs.writeFile(filepath, req.body, (err) => {
        if (err)
            return res.error(`Failed to save uploaded file: ${err}`)

        log.info(`[INFO] Uploaded file with filename ${filename}. Expires after ${prettyms(expire)}`)
        res.json({
            success: true,
            id: filename,
            type: ext,
            expire: expire,
            expireAt: Date.now() + expire,
        })

        deleteFile(filepath, filename, expire)
    })
})

app.get('/download', (req, res) => {
    var filename = req.query.id
    if (!filename)
        return res.error('Invalid id', 400)

    var mimetype = mime.lookup(filename)
    if (!mimetype)
        return res.error('Invalid id', 400)

    log.debug(`[DEBUG] Trying read file "${filename}"`)
    var filepath = path.join(STORE_PATH, filename)
    if (!fs.existsSync(filepath))
        return res.error('File not found', 404)

    fs.readFile(filepath, (err, data) => {
        if (err)
            return res.error(`Failed to read file: ${err.message}`)

        log.info(`[INFO] File "${filename}" successfully downloaded.`)
        res.set('Content-Type', mimetype)
        res.send(data)
    })
})

app.listen(PORT, () => {
    log.info(`[INFO] Running on http://localhost:${PORT}`)
})