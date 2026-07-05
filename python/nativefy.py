import socket
import json

class NativeUI:
    def __init__(self, host="127.0.0.1", port=9090):
        self.host = host
        self.port = port

    def _send_command(self, payload):
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.connect((self.host, self.port))
                s.sendall((json.dumps(payload) + "\n").encode("utf-8"))
                response = s.recv(1024)
                return json.loads(response.decode("utf-8"))
        except Exception as e:
            print(f"Error communicating with Native-fy: {e}")
            return None

    def reload(self):
        return self._send_command({"command": "reload"})

    def toggle_dashboard(self):
        return self._send_command({"command": "toggleDashboard"})

    def screenshot(self, path="screenshot.png"):
        return self._send_command({"command": "screenshot", "path": path})

if __name__ == "__main__":
    ui = NativeUI()
    print("Testing Native-fy Python Bindings...")
    print(ui.toggle_dashboard())
