import http.server, socketserver

class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        '.wasm': 'application/wasm',
    }

print('xguitar — http://localhost:8080')
socketserver.ThreadingTCPServer(('127.0.0.1', 8080), Handler).serve_forever()
