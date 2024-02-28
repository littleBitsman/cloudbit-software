import asyncio
import json
import subprocess
from enum import Enum
from typing import Any
from time import sleep

from websockets import client as websockets

# with open('/var/lb/mac') as file:
#     MAC_ADDRESS = file.readline()

# with open('/var/lb/id') as file:
#     CLOUDBIT_ID = file.readline()

MAC_ADDRESS = "testMac"
CLOUDBIT_ID = "testId"


def JSONEncode(dictionary: dict[str, Any]) -> str:
    return json.dumps(dictionary)


def JSONDecode(jsonString: str) -> dict[str, Any]:
    return json.loads(jsonString)


def read_ADC() -> int:
    try:
        output = subprocess.check_output(
            ['/usr/local/lb/ADC/bin/getADC', '-1'], universal_newlines=True)
        value = int(output.splitlines()[0])
        return value
    except BaseException:
        return 0


def write_DAC(value: int):
    try:
        hex_value = f'0x{value:04X}'
        # subprocess.check_call(['/usr/local/lb/ADC/bin/setDAC', hex_value])
        subprocess.run(['/usr/local/lb/ADC/bin/setDAC', hex_value])
    except BaseException:
        return


class LEDColors(Enum):
    RED = 'red'
    GREEN = 'green'
    BLUE = 'blue'
    YELLOW = 'yellow'
    TEAL = 'teal'
    PURPLE = 'purple'
    VIOLET = 'purple'
    WHITE = 'white'
    CLOWNBARF = 'clownbarf'


class LEDStatus(Enum):
    OFF = 'off'
    BLINK = 'blink'
    HOLD = 'hold'


def writeLED(str: str):
    try:
        # subprocess.check_call(['/usr/local/lb/LEDColor/bin/setColor', str])
        subprocess.run(['/usr/local/lb/LEDColor/bin/setColor', str])
    except BaseException:
        return

def setLED(color: LEDColors):
    writeLED(color.value)


def setLEDStatus(status: LEDStatus):
    writeLED(status.value)

Opcodes = {
    'INPUT': 0x1,
    'OUTPUT': 0x2,
    'HELLO': 0x3,
    'HEARTBEAT': 0x4,
    'HEARTBEAT_ACK': 0x5,
    'CLOWNBARF': 0x6
}


async def main():
    websocket = await websockets.connect(
        'ws://127.0.0.1:3000',
        extra_headers={
            'User-Agent': 'littleARCH cloudBit',
            'MAC-Address': MAC_ADDRESS,
            'CB-Id': CLOUDBIT_ID
        })

    async def send(data: dict[str, Any]):
        return await websocket.send(JSONEncode(data))

    async def hb(hbint): 
        while True:
            await asyncio.sleep(hbint / 1000)
            await send({'opcode': Opcodes['HEARTBEAT']})
        
    async def handle_message(message: str):
        msg = JSONDecode(message)
        if msg['opcode'] == Opcodes['HELLO']:
            asyncio.create_task(hb(msg['heartbeat_interval']))
        elif msg['opcode'] == Opcodes['OUTPUT']:
            write_DAC(msg['data']['value'])
        elif msg['opcode'] == Opcodes['CLOWNBARF']:
            setLED(LEDColors.CLOWNBARF)
    setLED(LEDColors.GREEN)
    setLEDStatus(LEDStatus.HOLD)
        
    currentInput = -1
    while True:
        try:
            message = await asyncio.wait_for(websocket.recv(), timeout=1)
            await handle_message(message)
        except BaseException:
            ""
        now = read_ADC()
        if now != currentInput:
            currentInput = now
            await send({'opcode': Opcodes['INPUT'], 'data': {'value': now}})
        await asyncio.sleep(0.5)

while True:
    try:
        asyncio.run(main())
    except BaseException:
        setLED(LEDColors.RED)
        setLEDStatus(LEDStatus.BLINK)
        sleep(2)