import asyncio
import signal
import websockets
import json
import logging
import time

async def foo_subscribe():
    url = 'wss://ws.okx.com:8443/ws/v5/public'
    json_data_subscribe = {
        "op": "subscribe",
        "args": [{
            "channel": "instruments",
            "instType": "SPOT"
        }]
    }
    json_data_unsubscribe = {
        "op": "unsubscribe",
        "args": [{
            "channel": "instruments",
            "instType": "SPOT"
        }]
    }

    PING_INTERVAL = 20  # Set the interval for sending 'ping' messages (in seconds)
    last_message_time = time.time()

    async def cleanup():
        # 发送取消订阅请求
        await websocket.send(json.dumps(json_data_unsubscribe))
        logging.info(f"Sent unsubscribe request: {json_data_unsubscribe}")

        # 关闭 WebSocket 连接
        await websocket.close()

    async def handle_messages():
        # 接收服务器的响应
        try:
            while True:
                response = await websocket.recv()

                # Reset the timer upon receiving a message

                if response == 'pong':
                    # Handle 'pong' response
                    # logging.info("Received 'pong' response")
                    continue
                
                logging.info(f"Received response from server: {response}")
                reset_timer()

                

        except websockets.exceptions.ConnectionClosed:
            logging.info("WebSocket connection closed")

    def reset_timer():
        nonlocal last_message_time
        last_message_time = time.time()

    async def ping():
        while True:
            # Send 'ping' every N seconds
            await asyncio.sleep(PING_INTERVAL)
            # logging.info("Sending 'ping'")
            await websocket.send('ping')

    

    try:
        async with websockets.connect(url) as websocket:
            # 将 JSON 数据转换为字符串并发送订阅请求到服务器
            await websocket.send(json.dumps(json_data_subscribe))
            logging.info(f"Sent subscribe request: {json_data_subscribe}")

            # 启动一个任务来处理消息
            messages_task = asyncio.create_task(handle_messages())

            # 启动定时发送 'ping' 任务
            ping_task = asyncio.create_task(ping())

            # 设置 Ctrl+C 信号处理器
            loop = asyncio.get_event_loop()
            loop.add_signal_handler(signal.SIGINT, lambda: loop.create_task(cleanup()))

            # 等待消息处理任务完成
            await messages_task
    except websockets.exceptions.ConnectionClosed:
        logging.info("WebSocket connection closed")

logging.basicConfig(filename='./okx.log', level=logging.INFO, format='%(asctime)s - %(message)s')
# 运行 WebSocket 客户端
asyncio.run(foo_subscribe())
