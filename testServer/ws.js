const ws = require('ws')

const server = new ws.Server({
    port: 3000
})

server.on('connection', (c, r) => {
    console.log('connect')
    c.on('message', d => console.log(d.toString('utf8')))
})

process.stdin.on('data', d => {
    const num = parseInt(d.toString('utf8'))
    if (isNaN(num) || num < 0 || num > 0xFFFF) return
    
    server.clients.forEach(v => v.send(JSON.stringify({
        opcode: 0x2,
        data: {
            value: num
        }
    })))
})