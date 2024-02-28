const dgram = require('node:dgram')
const { exit } = require('node:process')
const socket = dgram.createSocket('udp4')

socket.bind({
    port: 3000,
    address: 'localhost'
})

socket.on('message', (m, r) => {
    console.log(r)
    console.log(m.toString('utf8'))
})

process.stdin.on('data', d => {
    if (d.toString('utf8').includes('close')) return exit(0)
    const num = parseInt(d.toString('utf8'))
    if (isNaN(num) || num <= 0 || num >= 0xFFFF) return
    socket.send(JSON.stringify({
        opcode: 0x2,
        data: {
            value: num
        }
    }), 3001)
})