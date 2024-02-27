const ws = require('ws')

const server = new ws.Server({
    port: 2794
})

server.on('connection', c => {
    console.log('connect')
    c.on('message', d => {
        console.log(d.toString('utf8'))
    })
})

process.stdin.on('data', d => {
    const val = d.toString('utf8').trim()
    server.clients.forEach(c => c.send(val))
})