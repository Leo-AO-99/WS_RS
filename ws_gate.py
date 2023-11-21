import asyncio
import signal
import websockets
import json
import time
import logging

async def foo_subscribe():
    url = 'wss://api.gateio.ws/ws/v4/ann'
    json_data_subscribe = {
        "time": int(time.time()),
        "channel": "announcement.summary_listing",
        "event": "subscribe",
        "payload": ["en"]
    }
    json_data_unsubscribe = {
        "time": int(time.time()),
        "channel": "announcement.summary_listing",
        "event": "unsubscribe",
        "payload": ["en"]
    }

    async def cleanup():
        # 发送取消订阅请求
        json_data_unsubscribe['time'] = int(time.time())
        await websocket.send(json.dumps(json_data_unsubscribe))

        json_data_unsubscribe['time'] = int(time.time())
        json_data_unsubscribe['payload'].append('cn')
        await websocket.send(json.dumps(json_data_unsubscribe))
        # logging.info(f"Sent unsubscribe request: {json_data_unsubscribe}")

        # 关闭 WebSocket 连接
        await websocket.close()

    async def handle_messages():
        # 接收服务器的响应
        latest_code = 0
        try:
            while True:
                response = await websocket.recv()
                data = json.loads(response)
                try:
                    url = data['result']['origin_url']
                    code = int(url.split('/')[-1])
                    if latest_code >= code:
                        continue
                    
                    latest_code = code
                    ts_ms = data['time_ms']
                    title = data['result']['title']
                    logging.info(f"time_ms: {ts_ms}, url: {url}, title: {title}")
                except KeyError: 
                    logging.info(json.dumps(data))
        except websockets.exceptions.ConnectionClosed:
            logging.info("WebSocket connection closed")

    try:
        async with websockets.connect(url) as websocket:
            # 将 JSON 数据转换为字符串并发送订阅请求到服务器
            await websocket.send(json.dumps(json_data_subscribe))
            logging.info(f"Sent subscribe request: {json_data_subscribe}")

            # 启动一个任务来处理消息
            messages_task = asyncio.create_task(handle_messages())

            # 设置 Ctrl+C 信号处理器
            loop = asyncio.get_event_loop()
            loop.add_signal_handler(signal.SIGINT, lambda: loop.create_task(cleanup()))

            # 等待消息处理任务完成
            await messages_task
    except websockets.exceptions.ConnectionClosed:
        logging.info("WebSocket connection closed")

logging.basicConfig(filename='./gate.log', level=logging.INFO, format='%(asctime)s - %(message)s')
# 运行 WebSocket 客户端
asyncio.run(foo_subscribe())
