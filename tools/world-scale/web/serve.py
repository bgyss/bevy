#!/usr/bin/env python3
"""Serve the generated world-scale WASM client with correct MIME types."""
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler

ThreadingHTTPServer(("127.0.0.1", 48000), SimpleHTTPRequestHandler).serve_forever()
