const ws = require('ws')

const server = new ws.Server({
    port: 3000
})

server.broadcast = function (d) {
    server.clients.forEach(c => c.send(d))
}

server.on('connection', (c, r) => {
    console.log('connect')
    console.log(r.headers)
    c.on('message', d => console.log(`d: ${d.toString('utf8')}`))
})

const opcodes = [0x2, 0xF1, 0xF3]
process.stdin.on('data', d => {
    const str = d.toString('utf8')
    const args = str.split(': ')
    const opcode = parseInt(args[0])
    if (!opcodes.includes(opcode)) return

    if (opcode == 0x2) {
        const num = parseInt()
        if (isNaN(num) || num < 0 || num > 0xFFFF) return

        server.broadcast(JSON.stringify({
            opcode: 0x2,
            data: {
                value: num
            }
        }))
    } else if (opcode == 0xF1) {
        console.log('not yet...')
    } else if (opcode == 0xF3) {
        server.broadcast(JSON.stringify({
            opcode: 0xF3
        }))
    }

    console.log('ba')
})