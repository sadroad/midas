class ReconnectingWebSocket {
  constructor(url, protocols, options) {
    this.url = url;
    this.protocols = protocols;
    this.options = options || {};
    this.readyState = WebSocket.CLOSED;
    this.onopen = () => {};
    this.onmessage = () => {};
    this.onerror = () => {};
    this.onclose = () => {};

    this.reconnectInterval = this.options.reconnectInterval || 1000;
    this.maxReconnectInterval = this.options.maxReconnectInterval || 30000;
    this.reconnectDecay = this.options.reconnectDecay || 1.5;
    this.timeoutInterval = this.options.timeoutInterval || 2000;
    this.bufferedAmount = 0;

    this.shouldReconnect = true;
    this.reconnectAttempts = 0;
    this.socket = null;
    this.forcedClose = false;

    this.connect();
  }

  connect() {
    this.socket = new WebSocket(this.url, this.protocols);
    this.readyState = WebSocket.CONNECTING;
    this.bufferedAmount = 0;

    this.socket.onopen = (event) => {
      console.log('ReconnectingWebSocket: Connected');
      this.readyState = WebSocket.OPEN;
      this.reconnectAttempts = 0;
      this.onopen(event);
    };

    this.socket.onmessage = (event) => {
      this.onmessage(event);
    };

    this.socket.onerror = (event) => {
      console.error('ReconnectingWebSocket: Error', event);
      this.onerror(event);
    };

    this.socket.onclose = (event) => {
      console.log(
        'ReconnectingWebSocket: Closed',
        event.code,
        event.reason,
        event.wasClean
      );
      this.readyState = WebSocket.CLOSED;

      if (this.shouldReconnect && !this.forcedClose) {
        const reconnectDelay =
          this.reconnectInterval * Math.pow(this.reconnectDecay, this.reconnectAttempts);
        const actualDelay = Math.min(reconnectDelay, this.maxReconnectInterval);

        console.log(
          `ReconnectingWebSocket: Attempting reconnection in ${actualDelay}ms`
        );
        setTimeout(() => {
          this.reconnectAttempts++;
          this.connect();
        }, actualDelay);
      }

      this.onclose(event);
    };
  }

  send(data) {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(data);
    } else {
      console.warn('ReconnectingWebSocket: Socket is not open. Message not sent.');
    }
  }

  close(code, reason) {
    this.forcedClose = true;
    if (this.socket) {
      this.socket.close(code, reason);
    }
  }

  refresh() {
    if (this.socket) {
      this.socket.close();
    }
  }
}

let initialConnect = true;

const hotReloadSocket = new ReconnectingWebSocket("ws://localhost:3000/_reload");

hotReloadSocket.onopen = () => {
  console.log('Hot reload WebSocket connected.');
  if (!initialConnect) {
    console.log('Hot reload detected! Page likely updated.');
    window.location.reload();
  }
  initialConnect = false;
};

hotReloadSocket.onclose = (event) => {
  console.log('Hot reload WebSocket closed.');
};
