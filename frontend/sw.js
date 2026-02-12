const CACHE_NAME = "vibetorrent-v2";
const ASSETS_TO_CACHE = [
  "/",
  "/index.html",
  "/manifest.json",
  "/icon-192.png",
  "/icon-512.png",
];

// Install event - cache assets
self.addEventListener("install", (event) => {
  console.log("[Service Worker] Installing...");
  event.waitUntil(
    caches
      .open(CACHE_NAME)
      .then((cache) => {
        console.log("[Service Worker] Caching static assets");
        return cache.addAll(ASSETS_TO_CACHE);
      })
      .then(() => {
        console.log("[Service Worker] Skip waiting");
        return self.skipWaiting();
      }),
  );
});

// Activate event - clean old caches
self.addEventListener("activate", (event) => {
  console.log("[Service Worker] Activating...");
  event.waitUntil(
    caches
      .keys()
      .then((cacheNames) => {
        return Promise.all(
          cacheNames.map((key) => {
            if (key !== CACHE_NAME) {
              console.log("[Service Worker] Deleting old cache:", key);
              return caches.delete(key);
            }
          }),
        );
      })
      .then(() => {
        console.log("[Service Worker] Claiming clients");
        return self.clients.claim();
      }),
  );
});

// Fetch event - network first for HTML, cache fallback for API calls
self.addEventListener("fetch", (event) => {
  const url = new URL(event.request.url);

  // Network-first strategy for API calls
  if (url.pathname.startsWith("/api/")) {
    event.respondWith(
      fetch(event.request).catch(() => {
        // Could return cached API response or offline page
        return new Response(JSON.stringify({ error: "Offline" }), {
          headers: { "Content-Type": "application/json" },
        });
      }),
    );
    return;
  }

  // Network-first strategy for HTML pages (entry points)
  // This ensures users always get the latest version of the app
  if (
    event.request.mode === "navigate" ||
    url.pathname.endsWith("index.html") ||
    url.pathname === "/"
  ) {
    event.respondWith(
      fetch(event.request)
        .then((response) => {
          // Cache the latest version of the HTML
          const responseToCache = response.clone();
          caches.open(CACHE_NAME).then((cache) => {
            cache.put(event.request, responseToCache);
          });
          return response;
        })
        .catch(() => {
          return caches.match(event.request);
        }),
    );
    return;
  }

  // Cache-first strategy for static assets (JS, CSS, Images)
  event.respondWith(
    caches.match(event.request).then((response) => {
      return (
        response ||
        fetch(event.request).then((fetchResponse) => {
          // Optionally cache new requests
          if (fetchResponse && fetchResponse.status === 200) {
            const responseToCache = fetchResponse.clone();
            caches.open(CACHE_NAME).then((cache) => {
              cache.put(event.request, responseToCache);
            });
          }
          return fetchResponse;
        })
      );
    }),
  );
});

// Notification click event - focus or open app
self.addEventListener("notificationclick", (event) => {
  console.log("[Service Worker] Notification clicked:", event.notification.tag);
  event.notification.close();

  event.waitUntil(
    clients
      .matchAll({ type: "window", includeUncontrolled: true })
      .then((clientList) => {
        // If app is already open, focus it
        for (let client of clientList) {
          if (client.url === "/" && "focus" in client) {
            return client.focus();
          }
        }
        // Otherwise open new window
        if (clients.openWindow) {
          return clients.openWindow("/");
        }
      }),
  );
});

// Push notification event
self.addEventListener("push", (event) => {
  console.log("[Service Worker] Push notification received");
  const data = event.data ? event.data.json() : {};

  const title = data.title || "VibeTorrent";
  const options = {
    body: data.body || "New notification",
    icon: data.icon || "/icon-192.png",
    badge: data.badge || "/icon-192.png",
    tag: data.tag || "vibetorrent-notification",
    requireInteraction: false,
    // iOS-specific: vibrate pattern (if supported)
    vibrate: [200, 100, 200],
    // Add data for notification click handling
    data: {
      url: data.url || "/",
      timestamp: Date.now(),
    },
  };

  console.log("[Service Worker] Showing notification:", title, options);

  event.waitUntil(self.registration.showNotification(title, options));
});
