window.addEventListener('load', async () => {
  const ws = new WebSocket("ws://localhost:8000/ws");
  ws.onopen = () => console.log('socket opened');
  ws.onmessage = m => console.log('got message', m.data.length, m.data);
  ws.onerror = e => console.log('connection error', e);
  ws.onclose = () => console.log('connection closed');
});
