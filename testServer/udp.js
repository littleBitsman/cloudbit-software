const dgram = require('node:dgram')
const { exit } = require('node:process')
const socket = dgram.createSocket('udp4')

socket.bind({
    port: 3001,
    address: 'localhost'
})

const cbs = []

socket.on('message', (m, r) => {
    const d = m.toString('utf8')
    console.log(m)
    if (!(d.includes('I') && d.includes('O'))) {
        const mac = m.subarray(0, 6)
        if (cbs.find(v => mac.equals(v.mac))) return
        cbs.push({
            mac,
            port: r.port
        })
        console.log(cbs[cbs.length - 1])
    }
})

function clamp(num, min, max) {
    return Math.min(Math.max(num, min), max)
}

// 73 -> "I", 79 -> "O"
process.stdin.on('data', d => {
    if (d.toString('utf8').includes('close')) return exit(0)
    const num = clamp(parseInt(d.toString('utf8').trim()), 0, 0xFFFF)
    if (isNaN(num)) return
    const data = Buffer.from([0, 0, 0])
    data.writeUInt8(79)
    data.writeUInt16LE(num, 1)
    cbs.forEach(v => {
        socket.send(Buffer.concat([v.mac, data]), v.port)
    })
})