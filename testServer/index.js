const dgram = require('node:dgram')
const { exit } = require('node:process')
const socket = dgram.createSocket('udp4')

socket.bind({
    port: 3000,
    address: 'localhost'
})

const macs = []

socket.on('message', (m, r) => {
    console.log(r)
    console.log(m.toString('utf8'))
    try {
        const json = JSON.parse(m.toString("utf-8"))
        if (json.opcode == 0x3) {
            console.log(json)
            macs.push(json.mac.replaceAll(':', ''))
        }
    } catch { }
})

process.stdin.on('data', d => {
    if (d.toString('utf8').includes('close')) return exit(0)
    const num = parseInt(d.toString('utf8'))
    if (isNaN(num) || num < 0 || num >= 0xFFFF) return
    macs.forEach(v =>
        socket.send(Buffer.from(`${v}output:${num}`), 3001)
    )
})