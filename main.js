class Chat {
  constructor(ws, input, output) {
    this.ws = ws;
    this.input = input;
    this.output = output;
    this.addListeners();
  }

  addListeners() {
    this.ws.addEventListener('open', this.onOpen);
    this.ws.addEventListener('message', this.onMessage);
    this.ws.addEventListener('error', this.onError);
    this.ws.addEventListener('close', this.onClose);
    this.input.addEventListener('keydown', this.onKeyDown);
  }

  onOpen = () => console.log('socket opened');
  onMessage = message => {
    this.showMessage(message.data);
  };
  onError = error => {
    this.showMessage('connection error, check console for more details');
    console.log('connection error', error);
  };
  onClose = () => {
    this.showMessage('connection closed');
  };
  onKeyDown = event => {
    if (event.code === 'Enter') {
      event.preventDefault();
      this.sendMessage();
    }
  };

  sendMessage() {
    this.ws.send(this.input.value);
    this.input.value = '';
  }
  showMessage(text) {
    this.output.value += text + '\n';
  }
}

window.addEventListener('load', async () => {
  const ws = new WebSocket("ws://localhost:8081/ws");
  const input = document.getElementById('input');
  const output = document.getElementById('output');
  const chat = new Chat(ws, input, output);
});
