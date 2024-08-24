const ws = require('ws')
const bytes = require('bytes')

const server = new ws.Server({
    port: 80
})

server.broadcast = function (d) {
    server.clients.forEach(c => c.send(d))
}

server.on('connection', (c, r) => {
    console.log('connect')
    console.log(r.headers)
    const MAC = r.headers['mac-address'].toString()
    c.on('message', d => {
        try {
            const json = JSON.parse(d.toString('utf8'))
            const OPCODE = json.opcode
            if (OPCODE == 0x1) { // Input
                console.log(`INPUT ${MAC}:`)
                console.log(`- VALUE: ${json.data.value}`)
            } else if (OPCODE == 0x3) { // Identify thing
                console.log(`IDENTIFY:`)
                console.log(`- MAC: ${json.mac_address}`)
                console.log(`- ID: ${json.cb_id}`)
            } else if (OPCODE == 0xF4) {
                const stats = json.stats
                console.log(`STAT ${MAC}:`)
                console.log(`- CPU USAGE: ${stats.cpu_usage}%`)
                console.log(`- MEMORY USED: ${bytes.format(stats.memory_usage)}`)
                console.log(`- TOTAL MEMORY: ${bytes.format(stats.total_memory)}`)
                console.log(`- MEMORY USED (%): ${stats.memory_usage_percent}%`)
                console.log(`- CPU TEMP (K): ${stats.cpu_temp_kelvin}`)
            } else {
                console.log(`invalid opcode; packet: ${d.toString('utf8')}`)
            }
        } catch {}
    })
})

const opcodes = [0x2, 0xF1, 0xF3]
process.stdin.on('data', d => {
    const str = d.toString('utf8')
    const args = str.split(': ')
    const opcode = parseInt(args[0])
    if (!opcodes.includes(opcode)) return

    if (opcode == 0x2) {
        const num = parseInt(args[1])
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