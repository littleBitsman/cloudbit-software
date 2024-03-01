const dgram = require('node:dgram')
const { exit } = require('node:process')
const socket = dgram.createSocket('udp4')

socket.bind({
    port: 3000,
    address: 'localhost'
})

const data = []

socket.on('message', (m, r) => {
    const d = m.toString('utf8')
    console.log(d)
    if (d.endsWith('identify')) {
        const mac = d.replace('identify', '')
        if (data.find(v => v.mac == mac)) return
        data.push({
            mac,
            port: r.port
        })
    }
})

process.stdin.on('data', d => {
    if (d.toString('utf8').includes('close')) return exit(0)
    const num = parseInt(d.toString('utf8'))
    if (isNaN(num) || num < 0 || num > 0xFFFF) return
    data.forEach(v => socket.send(Buffer.from(`${v.mac}output:${num}`), v.port))
})