/**
 * Apex Service Worker — Thin Fetch Proxy
 *
 * Intercepts fetch requests to /apex/* and forwards them to the
 * Apex SharedWorker (apex-worker.js) which owns SQLite + WASM.
 *
 * This worker is stateless — if it dies and restarts, it just needs
 * the page to re-send it a MessageChannel port to the SharedWorker.
 */

// The MessagePort to the SharedWorker (set by the page via postMessage)
let workerPort = null;

// Pending requests waiting for the port to be established
let pendingPortRequests = [];

// ============================================================================
// Lifecycle
// ============================================================================

self.addEventListener("install", () => self.skipWaiting());
self.addEventListener("activate", (event) =>
  event.waitUntil(self.clients.claim()),
);

// ============================================================================
// Message handler — receives the SharedWorker port from the page
// ============================================================================

self.addEventListener("message", (event) => {
  if (event.data.type === "set-port") {
    // The port is transferred via postMessage(data, [port]) — it arrives in event.ports
    workerPort = event.ports[0];

    // Listen for responses from the SharedWorker on this port
    workerPort.onmessage = (evt) => {
      const { id, ...result } = evt.data;
      if (id && pendingResponses.has(id)) {
        const { resolve } = pendingResponses.get(id);
        pendingResponses.delete(id);
        resolve(result);
      }
    };

    // Flush any requests that arrived before the port was ready
    for (const pending of pendingPortRequests) {
      sendToWorker(pending.data).then(pending.resolve, pending.reject);
    }
    pendingPortRequests = [];
  }
});

// ============================================================================
// Fetch interceptor — forward /apex/* to the SharedWorker
// ============================================================================

self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);
  if (!url.pathname.startsWith("/apex/")) return;

  event.respondWith(handleApexFetch(url, event.request));
});

async function handleApexFetch(url, request) {
  const path = url.pathname.replace(/^\/apex\//, "");

  // Extract parameters
  let params = {};
  if (
    request.method === "POST" ||
    request.method === "PUT" ||
    request.method === "PATCH"
  ) {
    const body = await request.text();
    if (body) {
      try {
        params = JSON.parse(body);
      } catch {
        params = {};
      }
    }
  } else {
    for (const [key, value] of url.searchParams) {
      try {
        params[key] = JSON.parse(value);
      } catch {
        params[key] = value;
      }
    }
  }

  try {
    const result = await sendToWorker({
      type: "apex-fetch",
      path,
      method: request.method,
      params,
    });

    if (result.success) {
      const headers = { "Content-Type": "application/json" };
      if (result.cacheable) {
        headers["Cache-Control"] = "public, max-age=30";
      } else {
        headers["Cache-Control"] = "no-store";
      }
      return new Response(result.body, { status: result.status, headers });
    } else {
      return jsonResponse(result.status || 500, {
        error: result.error,
        availableRoutes: result.availableRoutes,
      });
    }
  } catch (err) {
    return jsonResponse(503, {
      error: "SharedWorker unavailable: " + err.message,
    });
  }
}

// ============================================================================
// Send a message to the SharedWorker and await the response
// ============================================================================

let msgId = 0;
const pendingResponses = new Map();

async function requestPortFromClients() {
  const clients = await self.clients.matchAll({ type: "window" });
  for (const client of clients) {
    client.postMessage({ type: "need-port" });
  }
}

function sendToWorker(data) {
  return new Promise((resolve, reject) => {
    if (!workerPort) {
      // Queue until the page sends us the port
      pendingPortRequests.push({
        data,
        resolve: (r) => resolve(r),
        reject: (e) => reject(e),
      });

      // Ask connected pages to re-send the bridge port
      requestPortFromClients();

      // If no port arrives within 5s, fail
      setTimeout(() => {
        reject(new Error("No SharedWorker port received"));
      }, 5000);
      return;
    }

    const id = ++msgId;
    pendingResponses.set(id, { resolve, reject });

    workerPort.postMessage({ ...data, id });

    // Timeout after 30s
    setTimeout(() => {
      if (pendingResponses.has(id)) {
        pendingResponses.delete(id);
        reject(new Error("SharedWorker request timed out"));
      }
    }, 30000);
  });
}

function jsonResponse(status, body) {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}
